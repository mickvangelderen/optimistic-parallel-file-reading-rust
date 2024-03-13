You are given a number of files and a message limit.
Each file contains some number of messages.
How would you gather all messages up to the limit in order if you want to minimize latency?
What if the file data is not on disk but retrieved over the network?
How do you make sure you do not open too many file or network handles at once?
What if determining the number of messages is almost as expensive as reading all of the messages?

At the time of writing, this repository contains a Rust program that spawns a thread pool (through rayon) and optimistically reads files until it realizes it has seen enough messages.
The author wonders if there is a better way to do things as we need to work around a lack of guarantees in which work is being picked up.
Can you make rayon pick up work (not complete, but start) in a given order?

## Example output

```txt
 1   ThreadId(2): sequential lines read = 0, total lines read = 0
 1   ThreadId(2): opening "b.txt"
0    ThreadId(3): sequential lines read = 0, total lines read = 0
0    ThreadId(3): opening "a.txt"
 1   ThreadId(2): closing "b.txt" with 4 lines read
  2  ThreadId(2): sequential lines read = 0, total lines read = 4
  2  ThreadId(2): opening "c.txt"
0    ThreadId(3): closing "a.txt" with 7 lines read
   3 ThreadId(3): sequential lines read = 11, total lines read = 11
   3 ThreadId(3): skipping "d.txt" because we already have enough data
  2  ThreadId(2): closing "c.txt" with 3 lines read
["a 1", "a 2", "a 3", "a 4", "a 5", "a 6", "a 7", "b 1", "b 2", "b 3", "b 4", "c 1", "c 2", "c 3"]
```
