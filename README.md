# Rust Mqtt Encoding & Decoding

### What is Mqtt?
MQTT is an ISO standard publish-subscribe-based messaging protocol. It works on top of the TCP/IP protocol.

### What is Rust?
Rust is a multi-paradigm systems programming language focused on safety, especially safe concurrency. Rust is syntactically similar to C++, but is designed to provide better memory safety while maintaining high performance.

### What is mqttrs?

It is library which can be used in any rust projects where you need to transform valid mqtt bytes buffer to Mqtt types and vice versa. 

In short it is encoding/decoding library which you can use it in sync as well as async environment.

The way it works is, It will take byte buffer as input and then will try to read the header of the mqtt packet, if the packet is not completely received as it happens in async networking, the library function will return `None` and will not remove any bytes from buffer.

Once, the whole mqtt packet is received, mqttrs will convert the bytes into appropriate mqtt packet type and return as well as remove all bytes from the beginning which belongs to already received packet.  

So, in this way, this library can be used for sync tcp streams as well as async streams like tokio tcp streams. 

