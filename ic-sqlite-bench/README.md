## ic-sqlite benchmark test comparison with the similar setup on ic-wasi-polyfill

## Summary

This is the benchmark from the (ic-sqlite)[] project running on the fresh rusqlite via ic-wasi-polyfill with the mounted memory storage setup.

.
#### [Performance benchmarks for SQL commands] comaring [ic-sqlite](https://github.com/froghub-io/ic-sqlite/) vs ic-wasi-polyfill

| SQL <br/> commands               | performance counter <br/> 1w single table data | performance counter <br/> 10w single table data | performance counter <br/> 50w single table data | performance counter <br/> 100w single table data |
|----------------------------------|------------------------------------------------|-------------------------------------------------|-------------------------------------------------|--------------------------------------------------|
| create table (icsqlite)          | 1194347                                        | 1433766                                         | 2565609                                         | 4066020                                          | 
| create table (ic-wasi-polyfill)  | 543435                                         | 569059                                          | 722663                                          | 1081202                                          | 
---
| create index <br/> (empty table) (icsqlite)         | 884588                      | 1122419                                         | 2241730                                         | 3601724                                          |
| create index <br/> (empty table) (ic-wasi-polyfill) | 499412                      | 521789                                          | 662403                                          | 786819                                           |
| count (icsqlite)                                    | 209847                      | 2995943                                         | 15183853                                        | 30392494                                         | 
| count (ic-wasi-polyfill)                            | 66443                       | 6682403                                         | 35484395                                        | 71440938                                         | 
| insert (icsqlite)                                   | 350256                      | 349635                                          | 351731                                          | 355381                                           | 
| insert (ic-wasi-polyfill)                           | 389373                      | 392164                                          | 391701                                          | 394134                                           | 
| select <br/> (where primary key) (icsqlite)         | 265363                      | 265960                                          | 265345                                          | 268112                                           | 
| select <br/> (where primary key) (ic-wasi-polyfill) | 77619                       | 80183                                           | 79592                                           | 80368                                            | 
| select <br/> (where index field) (icsqlite)         | 312389                      | 314594                                          | 314666                                          | 319276                                           | 
| select <br/> (where index field) (ic-wasi-polyfill) | 96422                       | 99701                                           | 100277                                          | 103120                                           | 
| select <br/> (where like field) (icsqlite)          | 178263088                   | 1784671532                                      | limit for single message execution              | limit for single message execution               | 
| select <br/> (where like field) (ic-wasi-polyfill)  | 11674181                    | 123746208                                       | 627625061                                       | 1257467787                                       | 
| update <br/> (where primary key) (icsqlite)         | 385492                      | 389192                                          | 391599                                          | 394111                                           | 
| update <br/> (where primary key) (ic-wasi-polyfill) | 446870                      | 451379                                          | 453642                                          | 454555                                           | 
| update <br/> (where index filed) (icsqlite)         | 239384                      | 237908                                          | 237993                                          | 240998                                           | 
| update <br/> (where index filed) (ic-wasi-polyfill) | 78599                       | 79776                                           | 80032                                           | 81046                                            | 
| delete <br/> (where primary key) (icsqlite)         | 429190                      | 259541                                          | 419615                                          | 423064                                           |
| delete <br/> (where primary key) (ic-wasi-polyfill) | 702943                      | 381121                                          | 694079                                          | 694053                                           |

