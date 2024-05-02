# Skribbl IO - Rust

My attempt in making a drawing and guessing game like Skribbl IO with its server written in Rust. I also took this opportunity to try dabbling with processing input and outputs as bytes instead of text.

## Binary Protocol

I implemented my own binary protocol to encode and decode data into typed arrays, a.k.a, vectors of bytes `Vec<u8>`. Here is an overview.

![skribbl-protocol](https://github.com/Ragudos/skribbl/assets/133567781/7beca070-534c-43dd-821c-8679b859dd7e)

`BINARY_PROTOCOL_VERSION` will help us know which parser to use. This will be usefull if this app scales (99% not lol), and we can't break the current protocol.
`Event` Useful to know if the server needs to parse the incoming data to release a different outcoming data. This is also used by the client to know which type a data is.
`DataLengthSpanInBytes` Since there might be data where its length is more than 255 (one byte), we need to know this.
`DataLength` The length of the data.
`Data` The data itself in bytes. The importance of the `DataLength` shines here as we'll only get the indeces or bytes after `DataLength` up to `DataLength - 1`.
