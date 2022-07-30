Clone Repository via git clone (then enter url)

cd into program folder

cargo build

cargo build-bpf

run solana-test-validator in another terminal and in yet another terminal, solana logs (to view msg's)
Make sure test-validator url is set to: http://127.0.0.1:8899 (see solana docs for changing cluster)

cargo build-bpf again then run the 'solana program deploy...so' command it recommends to deploy

cd into client folder and npm install

npm run test (make sure you have the solana-test-validator running per step 7)

// Notes
When running for the firsy time, ensure that the Instruction is set to 0 to create a Blog Account (see main.ts row 44)
Once a blog account is created, set it to 1 to make a post.
Each time you want to make a post, the POSTN variable in main.ts should be incremented (as slugs are part of the PDA account creation)

Special thanks to: https://solanacookbook.com/guides/account-maps.html#deriving-pdas
