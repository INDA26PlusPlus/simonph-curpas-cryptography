Tcp protokoll:

# Client sends data

```
{
    type:"send",
    file_id:str,
    ser_data:Vec<u8>
}
```

Server response if it added the file successfully

```
{
    status:"ok",
    mask:Vec<[u8;32]>,
    sides:Vec<bool>
}
```

# Client polls data

```
{
    type:"check",
    file_id:str
}
```

Server response if the file exists

```
{
    status:"ok"
}
```

# Client gets data

```
{
    type:"recieve",
    file_id:str
}
```

Server response if it found the file data successfully

```
{
    status:"ok",
    ser_data:Vec<u8>,
    mask:Vec<[u8;32]>,
    sides:Vec<bool>
}
```

# If the server fails any process, it instead responds with

```
{
    status:"failure",
    message:str
}
```