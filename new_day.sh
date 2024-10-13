#!/bin/bash
(set -a && . '.env' && set +a; cargo run --bin create_new_day $@)