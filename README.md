Step 1: Clone Repository via git clone (then enter url)

Step 2: cd into program folder

Step 3 cargo build

Step 4: cargo build-bpf

Step 5: cargo test

Step 6: cargo test-bpf

Step 7: run solana-test-validator in another terminal and in yet another terminal, solana logs (to view msg's)
Make sure test-validator url is set to: http://127.0.0.1:8899 (see solana docs for changing cluster)

Step 8: cargo build-bpf again then run the 'solana program deploy...so' command it recommends to deploy

Step 9: cd into client folder and npm install

Step 10: npm run test (make sure you have the solana-test-validator running per step 7)
