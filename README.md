# Skribbl IO - Rust

My attempt in making a drawing and guessing game like Skribbl IO with its server written in Rust. I also took this opportunity to try dabbling with processing input and outputs as bytes instead of text.

## Binary Protocol

I implemented my own binary protocol to encode and decode data into typed arrays, a.k.a, vectors of bytes `Vec<u8>`. Here is an overview.
