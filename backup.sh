#!/usr/bin/env bash

{ echo README.md 
  echo Cargo.toml 
  echo src/main.rs 
  echo templates/index.html.tera 
} | cpio -ao > depot.cpio

