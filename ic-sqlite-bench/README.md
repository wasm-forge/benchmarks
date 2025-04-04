## ic-sqlite benchmark test comparison with the similar setup on ic-wasi-polyfill

This is the benchmark from the [ic-sqlite](https://github.com/froghub-io/ic-sqlite/) project running on the fresh rusqlite and pragma settings similar to ic-sqlite via ic-wasi-polyfill with the mounted memory storage setup.

**Note:** The original ic-sqlite benchmarks were also re-run with the latest `dfx` to make sure they are objective, but they are now different from the ones reported in [ic-sqlite](https://github.com/froghub-io/ic-sqlite/).

#### Performance benchmarks for SQL commands: comaring ic-sqlite and ic-wasi-polyfill

| SQL <br/> commands               | performance counter <br/> 1w single table data | performance counter <br/> 10w single table data | performance counter <br/> 50w single table data | performance counter <br/> 100w single table data |
|----------------------------------|------------------------------------------------|-------------------------------------------------|-------------------------------------------------|--------------------------------------------------|
| create table (ic-sqlite)          | 333655                                        | 356917                                          | 496219                                          | 757636                                           | 
| create table (ic-wasi-polyfill)   | 289434                                        | 312745                                          | 439497                                          | 648166                                          | 
| create index <br/> (empty table) (ic-sqlite)         | 282802                     | 305614                                          | 426645                                          | 544042                                           |
| create index <br/> (empty table) (ic-wasi-polyfill)  | 244113                     | 266214                                          | 380644                                          | 503214                                           |
| count (ic-sqlite)                                    | 74223                      | 3671429                                         | 15753915                                        | 30849918                                         | 
| count (ic-wasi-polyfill)                             | 66630                      | 6162333                                         | 32712619                                        | 65857690                                         | 
| insert (ic-sqlite)                                   | 165777                     | 168657                                          | 167338                                          | 172585                                           | 
| insert (ic-wasi-polyfill)                            | 106986                     | 108953                                          | 110276                                          | 112531                                           | 
| select <br/> (where primary key) (ic-sqlite)         | 84140                      | 86268                                           | 86403                                           | 86842                                            | 
| select <br/> (where primary key) (ic-wasi-polyfill)  | 78300                      | 79691                                           | 79637                                           | 81044                                            | 
| select <br/> (where index field) (ic-sqlite)         | 103932                     | 108651                                          | 108901                                          | 109969                                           | 
| select <br/> (where index field) (ic-wasi-polyfill)  | 96667                      | 99857                                           | 100331                                          | 103092                                           | 
| select <br/> (where like field) (ic-sqlite)          | 11664070                   | 157402469                                       | 651191666                                       | 1268418927                                       | 
| select <br/> (where like field) (ic-wasi-polyfill)   | 11674245                   | 123189715                                       | 624715604                                       | 1251617977                                       | 
| update <br/> (where primary key) (ic-sqlite)         | 202902                     | 206925                                          | 210739                                          | 213028                                           | 
| update <br/> (where primary key) (ic-wasi-polyfill)  | 136962                     | 141263                                          | 143827                                          | 144974                                           | 
| update <br/> (where index filed) (ic-sqlite)         | 99598                      | 100654                                          | 101085                                          | 101695                                           | 
| update <br/> (where index filed) (ic-wasi-polyfill)  | 78008                      | 79352                                           | 79856                                           | 80394                                            | 
| delete <br/> (where primary key) (ic-sqlite)         | 328908                     | 155299                                          | 158216                                          | 158859                                           |
| delete <br/> (where primary key) (ic-wasi-polyfill)  | 238471                     | 99627                                           | 234341                                          | 234139                                           |

