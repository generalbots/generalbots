# Chapter 06: Rust Architecture Reference

This chapter covers the internal Rust architecture of BotServer, including its module structure, how to build from source, extend functionality with custom keywords, and add new dependencies.

BotServer is implemented as a single monolithic Rust application with multiple modules, not a multi-crate workspace. All functionality is contained within the `botserver` crate.