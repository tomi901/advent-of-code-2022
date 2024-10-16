use std::{fmt::{Debug, Display}, path::PathBuf, str::{FromStr, Lines}};
use cli_clipboard::{ClipboardContext, ClipboardProvider};

#[derive(Debug)]
enum Node {
    Dir(Dir),
    File {
        name: String,
        size: u64,
    },
}

impl Node {
    fn total_size(&self) -> u64 {
        match self {
            Node::Dir(dir) => dir.content.iter().map(Self::total_size).sum(),
            Node::File { size, .. } => *size,
        }
    }

    fn new_dir(name: &str) -> Self {
        Self::Dir(Dir::new(name))
    }

    fn new_file(name: &str, size: u64) -> Self {
        Self::File { name: name.to_string(), size }
    }

    fn as_dir(&self) -> Option<&Dir> {
        match self {
            Node::Dir(dir) => Some(dir),
            _ => None,
        }
    }

    fn display_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: u32) -> std::fmt::Result {
        for _ in 0..indent {
            write!(f, "  ")?;
        }
        write!(f, "- ")?;
        match self {
            Node::Dir(dir) => dir.display_fmt(f, indent + 1)?,
            Node::File { name, size } => {
                writeln!(f, "{} (file, size={})", name, size)?;
            },
        }
        Ok(())
    }
}

impl FromStr for Node {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split_whitespace();
        Ok(match split.next() {
            Some("dir") => {
                let name = split.next().expect("No name defined for dir");
                Self::new_dir(name)
            },
            Some(size_s) => {
                let size = size_s.parse().expect("Size has to be unsigned int");
                let name = split.next().expect("No name defined for file");
                Self::new_file(name, size)
            },
            None => panic!("Empty line"),
        })
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_indented(f, 0)
    }
}

#[derive(Debug)]
struct Dir {
    name: String,
    content: Vec<Node>,
}

impl Dir {
    fn new(name: &str) -> Self {
        Dir { name: name.to_string(), content: Default::default() }
    }

    fn from_commands(lines: &mut Lines) -> Self {
        let mut root_dir = Dir::new("/");
        let mut cur_path = PathBuf::from("/"); cur_path = PathBuf::from("/");
        
        let mut cur_line = lines.next();
        loop {
            let _cur_line = match cur_line {
                Some(l) => l,
                None => break,
            };

            let mut split = _cur_line.split_whitespace();
            assert_eq!(split.next(), Some("$"));
            match split.next() {
                Some("cd") => {
                    // println!("{}", _cur_line);
                    let path = split.next().expect("No path given");
                    if path == ".." { // We could have a chain of .. paths but the challenge doesn't require it
                        cur_path.pop();
                    } else {
                        cur_path.push(path);
                    }
                    // println!("Cur path: {:?}", cur_path);
                },
                Some("ls") => {
                    // println!("{}", _cur_line);
                    let dir = root_dir.get_dir_mut(&cur_path);
                    loop {
                        cur_line = lines.next();
                        let node = match cur_line {
                            Some(s) if s.starts_with("$") => break,
                            Some(s) => {
                                // println!("Found: {}", s);
                                s.parse::<Node>().expect("Failed to parse node")
                            },
                            None => break,
                        };
                        dir.content.push(node);
                    }
                    continue;
                },
                Some(cmd) => panic!("Unrecognized command: {}", cmd),
                None => panic!("No command given"),
            }

            cur_line = lines.next();
        }
        
        let mut cur_line = lines.next();
        loop {
            let _cur_line = match cur_line {
                Some(l) => l,
                None => break,
            };

            let mut split = _cur_line.split_whitespace();
            assert_eq!(split.next(), Some("$"));
            match split.next() {
                Some("cd") => {
                    // println!("{}", _cur_line);
                    let path = split.next().expect("No path given");
                    if path == ".." { // We could have a chain of .. paths but the challenge doesn't require it
                        cur_path.pop();
                    } else {
                        cur_path.push(path);
                    }
                    // println!("Cur path: {:?}", cur_path);
                },
                Some("ls") => {
                    // println!("{}", _cur_line);
                    let dir = root_dir.get_dir_mut(&cur_path);
                    loop {
                        cur_line = lines.next();
                        let node = match cur_line {
                            Some(s) if s.starts_with("$") => break,
                            Some(s) => {
                                // println!("Found: {}", s);
                                s.parse::<Node>().expect("Failed to parse node")
                            },
                            None => break,
                        };
                        dir.content.push(node);
                    }
                    continue;
                },
                Some(cmd) => panic!("Unrecognized command: {}", cmd),
                None => panic!("No command given"),
            }

            cur_line = lines.next();
        }
        root_dir
    }

    fn total_size(&self) -> u64 {
        self.content.iter().map(Node::total_size).sum()
    }

    fn get_dir_mut(&mut self, path: &PathBuf) -> &mut Self {
        let mut dir = self;
        for component in path.components() {
            if component.as_os_str() == "/" {
                continue;
            }
            dir = dir.content
                .iter_mut()
                .flat_map(|n| match n {
                    Node::Dir(dir) => (&dir.name[..] == component.as_os_str()).then_some(dir),
                    Node::File { .. } => None,
                })
                .next()
                .expect("Didn't found dir");
        }
        dir
    }

    fn display_fmt(&self, f: &mut std::fmt::Formatter<'_>, indent: u32) -> std::fmt::Result {
        writeln!(f, "{} (dir)", self.name)?;
        for node in self.content.iter() {
            node.display_indented(f, indent + 1)?;
        }
        Ok(())
    }

    fn get_deletable_dirs_size_repeated(&self, limit: u64) -> u64 {
        let self_size = self.total_size();
        let mut total_size = 0;
        if self_size <= limit {
            total_size += self_size;
        }

        total_size += self.content.iter()
            .flat_map(Node::as_dir)
            .map(|d| d.get_deletable_dirs_size_repeated(limit))
            .sum::<u64>();
        total_size
    }

    fn get_smallest_deletable_size(&self, limit: u64) -> Option<u64> {
        self.get_smallest_deletable_size_with_min(limit, None)
    }

    fn get_smallest_deletable_size_with_min(&self, limit: u64, min: Option<u64>) -> Option<u64> {
        let self_size = self.total_size();
        let mut current_min = min;
        if self_size >= limit && (current_min.is_none() || current_min.is_some_and(|min| self_size < min)) {
            println!("Found dir {} of size {}", self.name, self_size);
            current_min = Some(self_size);
        }

        for dir in self.content.iter().flat_map(Node::as_dir) {
            current_min = dir.get_smallest_deletable_size_with_min(limit, min);
        }
        current_min
    }
}

impl Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "- ")?;
        self.display_fmt(f, 0)
    }
}

fn main() {
    // part_1();
    part_2();
}

fn part_1() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let root_dir = Dir::from_commands(&mut input.lines());

    // println!("Folder structure:");
    // println!("{}", &root_dir);

    let result = root_dir.get_deletable_dirs_size_repeated(100_000);
    display_result(&result);
}

fn part_2() {
    let input = std::fs::read_to_string("./input.txt").expect("Error reading input file.");
    let root_dir = Dir::from_commands(&mut input.lines());

    const TOTAL_SIZE: u64 = 70_000_000;
    const REQUIRED_SIZE: u64 = 30_000_000;
    let occupied = root_dir.total_size();
    let current_free = TOTAL_SIZE - occupied;
    let to_delete: u64 = REQUIRED_SIZE - current_free;

    println!("{} bytes occupied", occupied);
    println!("{} bytes free", current_free);
    println!("{} to delete", to_delete);

    let result = root_dir.get_smallest_deletable_size(to_delete);
    display_result(&result);
}

// TODO: Move this to common library crate
fn display_result<T: Debug>(result: &T) {
    println!("Result:");
    let str_result = format!("{:?}", result);
    println!("{}", &str_result);

    let mut clipboard = ClipboardContext::new().unwrap();
    clipboard.set_contents(str_result.clone()).unwrap();
    println!("Copied result to clipboard!");
}
