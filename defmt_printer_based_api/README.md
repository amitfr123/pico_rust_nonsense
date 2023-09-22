# Defmt printer based api
bad name i know

## Unstable
This crate is based on the unstable api of defmt-decoder and therefore should not be consumed by other tools. 

This crate does not guaranty stability or support for older\newer defmt versions (if requested i will try to update it if there are new breaking changes).

If this crate works for you then fantastic happy to help. If not please don't create issues in the official defmt repository because Knurling-rs is unrelated to this crate and even wrote not to base tools on the unstable api.

## Should you use this crate?
You should use the existing tooling provided and supported by Knurling if possible.
If existing tooling do not match your needs you can use this crate to embed the defmt-printer logic in your project. 
FYI this crate is super simple so it should be easy to write your own better version of this crate (if you do link it and i will redirect to it).

## Why does this crate exists?
At the time of writing this crate the there are 2 ways (that i found) to enjoy the defmt utilities:

1. Use existing printers: probe-run, defmt-print, qemu-run.
2. Create custom tooling based on the unstable decoder api.

While working on a project for several reasons the first option was not available for me and to make my life easier i created a module based on the defmt-print that allows me to embed the printer logic within my code. After polishing it a little it seemed to me like it should have its own crate.