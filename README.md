## snipe

Snipe is a blocktime estimator for Ethereum written in Rust. Fair warning, I am 
just starting to learn Rust, so please critique and rip apart the code as you see fit. 

### Design 

There are a few functionalities available through the CLI.  

- Block to time 

Given a block number this function will return either the time at which the block occured 
or the time at which the block is expected to occur if the block number has not yet been 
reached.  Time estimation assumes that no slots are skipped.  

- Time to block 

Given a timestamp, return the first block to occur after that timestamp.  If the user 
specifies the year 2016, the first block number occuring on January 1, 2016 will be returned. 
This function optimistically assumes that the user means a block occuring after the Genesis block. 
This means that if a user specifies simply the year 2015, the function will return the genesis block. 
However if the user specifies a timestamp that cannot be turned into a block occuring at or before 
genesis, the program will error.

- Countdown 

This function provides a live updating countdown to the specified block number.

- Timezones 

List all timezones available to the user 


TODO 
add testing to make sure timezone management is done correctly and that 
the genesis / 01 conversion is still managed properly.  

implement binary search for block, return 

done
