## ic-sqlite benchmark test comparison with the similar setup on ic-wasi-polyfill

## Summary

This is the benchmark from the (ic-sqlite)[] project running on the fresh rusqlite via ic-wasi-polyfill with the mounted memory storage setup.

*Note:* The original ic-sqlite benchmarks were also re-run with the latest `dfx` to make sure they are objective, but they are now different from the ones reported in [ic-sqlite](https://github.com/froghub-io/ic-sqlite/).

#### [Performance benchmarks for SQL commands] comaring [ic-sqlite](https://github.com/froghub-io/ic-sqlite/) vs ic-wasi-polyfill

| SQL <br/> commands               | performance counter <br/> 1w single table data | performance counter <br/> 10w single table data | performance counter <br/> 50w single table data | performance counter <br/> 100w single table data |
|----------------------------------|------------------------------------------------|-------------------------------------------------|-------------------------------------------------|--------------------------------------------------|
| create table (ic-sqlite)          | 333655                                        | 356917                                          | 496219                                          | 757636                                           | 
| create table (ic-wasi-polyfill)   | 543435                                        | 569059                                          | 722663                                          | 1081202                                          | 
| create index <br/> (empty table) (ic-sqlite)         | 282802                     | 305614                                          | 426645                                          | 544042                                           |
| create index <br/> (empty table) (ic-wasi-polyfill)  | 499412                     | 521789                                          | 662403                                          | 786819                                           |
| count (ic-sqlite)                                    | 74223                      | 3671429                                         | 15753915                                        | 30849918                                         | 
| count (ic-wasi-polyfill)                             | 66443                      | 6682403                                         | 35484395                                        | 71440938                                         | 
| insert (ic-sqlite)                                   | 165777                     | 168657                                          | 167338                                          | 172585                                           | 
| insert (ic-wasi-polyfill)                            | 389373                     | 392164                                          | 391701                                          | 394134                                           | 
| select <br/> (where primary key) (ic-sqlite)         | 84140                      | 86268                                           | 86403                                           | 86842                                            | 
| select <br/> (where primary key) (ic-wasi-polyfill)  | 77619                      | 80183                                           | 79592                                           | 80368                                            | 
| select <br/> (where index field) (ic-sqlite)         | 103932                     | 108651                                          | 108901                                          | 109969                                           | 
| select <br/> (where index field) (ic-wasi-polyfill)  | 96422                      | 99701                                           | 100277                                          | 103120                                           | 
| select <br/> (where like field) (ic-sqlite)          | 11664070                   | 157402469                                       | 651191666                                       | 1268418927                                       | 
| select <br/> (where like field) (ic-wasi-polyfill)   | 11674181                   | 123746208                                       | 627625061                                       | 1257467787                                       | 
| update <br/> (where primary key) (ic-sqlite)         | 202902                     | 206925                                          | 210739                                          | 213028                                           | 
| update <br/> (where primary key) (ic-wasi-polyfill)  | 446870                     | 451379                                          | 453642                                          | 454555                                           | 
| update <br/> (where index filed) (ic-sqlite)         | 99598                      | 100654                                          | 101085                                          | 101695                                           | 
| update <br/> (where index filed) (ic-wasi-polyfill)  | 78599                      | 79776                                           | 80032                                           | 81046                                            | 
| delete <br/> (where primary key) (ic-sqlite)         | 328908                     | 155299                                          | 158216                                          | 158859                                           |
| delete <br/> (where primary key) (ic-wasi-polyfill)  | 702943                     | 381121                                          | 694079                                          | 694053                                           |

