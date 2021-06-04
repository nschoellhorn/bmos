#!/usr/bin/env zsh
rust-gdb --eval-command="target remote localhost:1234" --eval-command="set pagination off" target/x86_64-bmos/debug/bmos
