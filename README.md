# btc-2024-hack

For this hackathon I produced the stuff in Yellow and Green, Mara has provided the the stuff in Red.

![Screenshot 2024-07-16 at 5 22 41 PM](https://github.com/user-attachments/assets/23193cba-df98-43a2-a8d8-7951f684300c)


Using docker to run the coordinate node from [here](https://github.com/AnduroHackathon/coordinate-node/blob/main/Dockerfile)

Docker run cmd:
`docker run -v ~/.marachain:/root/.coordinate -p 18332:18332 -p 18333:18333 -p 27000:27000 --name sidechainnode andurot -testnet=1 -rpcuser=$USER -rpcpassword=$PASSWORD -server=1 -port=18333 -rpcport=18332 -rpcbind=0.0.0.0 -rpcallowip=0.0.0.0/0 -zmqpubrawblock=tcp://0.0.0.0:27000 -zmqpubrawtx=tcp://0.0.0.0:27000 -zmqpubhashtx=tcp://0.0.0.0:27000 -zmqpubhashblock=tcp://0.0.0.0:27000 -txindex=1`

## This is the website
![Screenshot 2024-07-16 at 5 26 13 PM](https://github.com/user-attachments/assets/2d7ed95d-2b1f-46f8-98fb-513208596053)

## Screenshot of Coordinate Node's logs that I'm running for voucher service
![Screenshot 2024-07-16 at 5 28 42 PM](https://github.com/user-attachments/assets/4ac60727-53bc-4902-804f-b8b8db91f410)

## Rust for my web service
<img width="606" alt="Screenshot 2024-07-16 at 5 34 08 PM" src="https://github.com/user-attachments/assets/6812d9ab-5f05-4e36-9e8e-f18779f058a2">
