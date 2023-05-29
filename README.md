# pg_chainutils: blockchain utilities for PostgreSQL

> Proof of Concept - expect bugs and breaking changes

pg_chainutils adds some simple utilities that help parse/transform/interact with blockchain datatypes and standards directly inside PostgreSQL. Contributions welcome!

Check out [pg_chainsync](https://github.com/usagi-coffee/pg_chainsync) extension to fetch blocks and events inside your database.

This extension is created using [pgrx](https://github.com/tcdi/pgrx)

## Utilities

- [X] H256 -> H160
- [ ] ...

### ERC20

- [X] Transfer
- [ ] Approval
- [ ] ...

### ERC721

- [X] Transfer
- [ ] ...

### Sushiswap

- [X] Swap
- [X] Sync
- [ ] ...

## Usage

```sql
CREATE EXTENSION pg_chainutils;
```

```sql
SELECT H160.from_H256("0x0000000000000000000000001111111111111111111111111111111111111111")
-- 0x1111111111111111111111111111111111111111
```

### ERC20 / ERC721

```sql
-- Takes abi hex encoded topics as argument
SELECT ERC20.transfer_from('{0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef,0x0000000000000000000000001111111111111111111111111111111111111111,0x0000000000000000000000002222222222222222222222222222222222222222}');
-- 0x1111111111111111111111111111111111111111

-- Takes abi hex encoded topics as argument
SELECT ERC20.transfer_to('{0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef,0x0000000000000000000000001111111111111111111111111111111111111111,0x0000000000000000000000002222222222222222222222222222222222222222}');
-- 0x2222222222222222222222222222222222222222

-- Takes log data as argument
SELECT ERC20.transfer_value('00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000001d688');
-- 120456

SELECT ERC721.transfer_token('00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000001d688');
-- 120456
```


## License

```
MIT License

Copyright (c) 2023 Kamil Jakubus and contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```