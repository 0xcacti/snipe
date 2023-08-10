## snipe

Snipe is a blocktime estimator for Ethereum written in Rust. Fair warning, I am 
just starting to learn Rust, so please critique and rip apart the code as you see fit. 

### Design 

The project is implemented as a binary crate CLI tool that calls into a library crate.
The CLI implements all the public methods. 

### CLI 

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


- Timezones 

List all timezones available to the user 

### Potential Improvements 

#### Better Error Handling 
As it stands, I am using unwraps on options and anyhow for results.  If there was
decent public interest in this tool, I would go through the effort to convert it 
to use thiserror in the library code.  

#### Search Optimization
Right now, the search algorithm is a simple binary search.  I may add checkpoints 
in the future, to make search distances significantly shorter. 

#### Countdown 
Add a live updating countdown in the terminal to the next block

#### Transaction Sniping 
Implement functionality that allows people to submit transactions with snipe 
and it will attempt to land the transaction in the given block or as close 
to the given time as possible.
