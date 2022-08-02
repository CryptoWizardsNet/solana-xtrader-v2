<html>
<h2>How to Run - Program Build</h2>

<p>After cloning the repo, cd into the Program folder.</p>
<p>Update the Cargo.toml file to represent a lib name you like.</p>
<p>Run Cargo Buil and Cargo Build BPF.</p>
<p>Run the Deploy code presented as solana program deploy ... etc as show in the terminal after the prior step.</p>

<h3>Important:</h3>
<p>You will likely run into issues if running on localhost (http://127.0.0.1:8899). Runnin on the devnet cluster https://api.devnet.solana.com is recommended. E.g: solana config set --url https://api.devnet.solana.com.</p>
<p>You will need to replace Clock.get(), Rent.get() and chainlink calls in the Program processor file if
wanting to continue on localhost.
</p>
<p>Remember that this method of trading will not allow you to claim until the time needed has past.</p>
<p>Therefore, remove the if statement Guard for that too if you want to test in real time.</p>

<h2>How to Run- Client Build</h2>

<p>Cd into the client folder.</p>
<p>npm install</p>
<p>npm run wallet maker (This creates a wallet for the Market Maker)</p>
<p>npm run wallet taker (This creates a wallet for the Market Taker and also Claimer as taker is claiming)</p>
<p>Ensure both maker and taker have at least 2.0 SOL</p>
<p>npm run account-create maker</p>
<p>npm run account-create taker</p>
<p>npm run account-fund maker</p>
<p>npm run account-fund taker</p>
<p>npm run maker (represents maker placing a trade)</p>
<p>Paste the tradeAccount address that is printed out on the taker.ts and claim.ts fields near the top of each of the two files.</p>
<p>This represents the Open Order set by the Maker that can be filled. Once it is filled by the Taker and the designated time has past, it can be claimed.</p>
<p>npm run taker (represents taker accepting the trade)</p>
<p>npm run claim</p>
</html>
