# Usage
First, run the libdatachannel side.
```
make
./libdatachannel-side
```

Then, run the str0m side in another terminal.
```
cd str0m-side
cargo run
```

Paste the offer printed by the libdatachannel side into the str0m side.
Paste the answer by the str0m side into the libdatachannel side.
1. Observe that `channel->isOpen()` remains false (on the libdatachannel side).

2. Observe that no ChannelOpened event is received for the datachannel opened by libdatachannel (on the str0m side).
