# README #

Rust clone of [Kai Uwe Rommels' NETIO Benchmark](http://www.ars.de/ars/ars.nsf/docs/netio) that I've always greatly enjoyed using, but
that (imho) has a couple downsides:

*   It was Not Invented Here (obviously)
*   The server side output differs from the client side and is far less readable
*   The license prevents it from ever hitting any repos (and Kai does not provide his own)

And because I wanted to take a look at [Rust](rust-lang.org) anyway, I thought I'd go ahead and write a clone for fun and great justice.

# Features #

* Nice tabular output on both client and server side
* IPv6 ready
* Tests with extremely small packets (like those of an SSH connection when someone's *extremely* good at typing)
* binaries for Windows and Linux available (64 bit `--release` builds)

# Installation #

## Linux

```
wget -O /usr/local/bin/netio https://github.com/Svedrin/netio/releases/download/v0.3.2/netio && chmod +x /usr/local/bin/netio
```

## Windows

Install the [Visual C++ Redistributable for Visual Studio 2015](https://www.microsoft.com/de-de/download/details.aspx?id=48145).

The download for the Windows build is currently unavailable.



# Usage Examples #

Here's how to use it. Server side:

```
root@hive:# netio -1
TCP server listening.
New connection from V6([2001:6f8:108f:1:c0c6:73c3:8b5:214c]:40704).
Packet size    32 bytes:   18.40 MBit/s Rx       21.40 MBit/s Tx
Packet size    64 bytes:   18.16 MBit/s Rx       20.87 MBit/s Tx
Packet size  1024 bytes:   18.59 MBit/s Rx       19.48 MBit/s Tx
Packet size  1492 bytes:   17.90 MBit/s Rx       20.32 MBit/s Tx
Packet size  1500 bytes:   18.79 MBit/s Rx       19.68 MBit/s Tx
Packet size  2048 bytes:   19.46 MBit/s Rx       20.25 MBit/s Tx
Packet size 16384 bytes:   18.53 MBit/s Rx       14.21 MBit/s Tx
Test finished.
```

Client side:

```
root@dev-psql-n1:~# netio 192.168.42.10
Connected to V4(192.168.42.10:55455).
Packet size    32 bytes:  422.29 MBit/s Tx      406.65 MBit/s Rx
Packet size    64 bytes:  827.98 MBit/s Tx      804.86 MBit/s Rx
Packet size  1024 bytes:    9.38 GBit/s Tx        6.83 GBit/s Rx
Packet size  1492 bytes:    9.38 GBit/s Tx        9.40 GBit/s Rx
Packet size  1500 bytes:    9.38 GBit/s Tx        9.39 GBit/s Rx
Packet size  2048 bytes:    9.38 GBit/s Tx        9.40 GBit/s Rx
Packet size 16384 bytes:    9.39 GBit/s Tx        9.40 GBit/s Rx
Test finished.
```

# Firewalls #

Netio uses TCP port 55455. The client always connects to the server, and then traffic flows
in both ways. This way you can easily test your internet connection by running the server
on an online node and the client on your notebook.

# Building from Source #

```
git clone https://github.com/Svedrin/netio.git
cd netio
cargo build --release
```

Note that the debug build seems to only reach throughput rates around 1.3Gbit/s, so if you're
going to benchmark a 10GE link, you *need* to build netio with `--release`.
