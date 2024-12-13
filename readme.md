### dnstt-tester
针对隧道工具的信息泄露自动化工具
#### build
create bin
```
cargo build -r
```
create logger files 
```
touch stdin stdout stderr
```
logger files name can be change,but should be given by args  ：
```
  --in <STDIN_FILE>
  --out <STDOUT_FILE>
  --err <STDERR_FILE>
```

##### usage
###### Client side
```
usage: tunnel-tool-tester.exe client [OPTIONS] --port <PORT> --exe <EXE> --args <ARGS> --reconnect-time-second <RECONNECT_TIME_SECOND> --conn-time-second <CONN_TIME_SECOND> --make-file-second <MAKE_FILE_SECOND> --file-size-range <FILE_SIZE_RANGE> --in <STDIN_FILE> --out <STDOUT_FILE> --err <STDERR_FILE> 

Options:
  -p, --port <PORT>                                    Ports this tool will connect to
      --bind <BIND>                                    [default: 127.0.0.1] IP address this tool will connect to
  -e, --exe <EXE>                                      The bin of tunnel tool
  -a, --args <ARGS>                                    the args of tunnel tool
  -r, --reconnect-time-second <RECONNECT_TIME_SECOND>  the time to restart tunnel tool wen it panic
  -c, --conn-time-second <CONN_TIME_SECOND>            after this time tunnel tool will be kill and restart
  -m, --make-file-second <MAKE_FILE_SECOND>            the interval of Generate and send random files （as bin stream）
  -f, --file-size-range <FILE_SIZE_RANGE>              size randge of random file ，such as 30~200 is min 30Bytes and max 200Bytes
      --in <STDIN_FILE>                                [default: stdin] write to child's stdin and close it when child start 
      --out <STDOUT_FILE>                              [default: stdout] the stdout of child
      --err <STDERR_FILE>                              [default: stderr] the stderr of child
      --no_stdin                                       don't copy and close stdin
  -h, --help                                           Print help

```
example  

when I want to run `./bin/dnstt-client` and give args `-doh https://doh.pub/dns-query -pubkey-file ./bin/server.pub *******(my domain) 127.0.0.1:&[port]` and run this program with args `-r 60 -m 25 -f 30~2000 -c 2 ` on port `6666`

```
./tunnel-tool-tester client --port 6666 --exe ./bin/dnstt-client -a "-doh https://doh.pub/dns-query -pubkey-file ./bin/server.pub *******(my domain) 127.0.0.1:&[port]" -r 60 -m 25 -f 30~2000 -c 2 
```
###### Server side
```
Usage: tunnel-tool-tester.exe server [OPTIONS] --port <PORT> --exe <EXE> --args <ARGS> --in <STDIN_FILE> --out <STDOUT_FILE> --err <STDERR_FILE>

Options:
  -p, --port <PORT>                                    the val of &[ports] on <ARGS>
      --ports <PORTS>                                  [default: ] the ports this tool will listen
  -e, --exe <EXE>                                      The bin of tunnel tool
      --bind <BIND>                                    [default: 127.0.0.1] the IP address this tool will bind to listen
  -a, --args <ARGS>                                     the args of tunnel tool
      --in <STDIN_FILE>                                [default: stdin] write to child's stdin and close it when child start 
      --out <STDOUT_FILE>                              [default: stdout] the stdout of child
      --err <STDERR_FILE>                              [default: stderr] the stderr of child
  -h, --help               Print help

```