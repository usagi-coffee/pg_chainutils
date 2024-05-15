# pg_chainutils: blockchain utilities for PostgreSQL

pg_chainutils adds some simple utilities that help parse/transform/interact with blockchain datatypes and standards directly inside PostgreSQL. 

Contributions and suggestions welcome!

This extension is created using [pgrx](https://github.com/tcdi/pgrx)
Check out [pg_chainsync](https://github.com/usagi-coffee/pg_chainsync) extension to fetch blocks and events inside your database.

## Usage

```sql
CREATE EXTENSION pg_chainutils;
```

### H256 / H160

```sql
SELECT H256.parse("0000000000000000000000001111111111111111111111111111111111111111");
-- 0x0000000000000000000000001111111111111111111111111111111111111111

SELECT H256.parse_slice(
    "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000a16e02e87b7454126e5e10d957a927a7f5b5d2be",
    64, 128
);
-- 0x000000000000000000000000a16e02e87b7454126e5e10d957a927a7f5b5d2be

SELECT H160.from_H256("0x0000000000000000000000001111111111111111111111111111111111111111")
-- 0x1111111111111111111111111111111111111111

SELECT H256.from_event("Sync(uint112,uint112)"); -- keccak256 of event signature
-- 0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1
```

### ERC20 / ERC721

```sql
-- Get abi encoded event hashes
SELECT ERC20.transfer_abi();
-- 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef

-- Takes abi hex encoded topics as argument
SELECT ERC20.transfer_from('{0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef,0x0000000000000000000000001111111111111111111111111111111111111111,0x0000000000000000000000002222222222222222222222222222222222222222}');
-- 0x1111111111111111111111111111111111111111
-- Or you can just use H160.from_H256 and pass second element of topics array

-- Takes abi hex encoded topics as argument
SELECT ERC20.transfer_to('{0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef,0x0000000000000000000000001111111111111111111111111111111111111111,0x0000000000000000000000002222222222222222222222222222222222222222}');
-- 0x2222222222222222222222222222222222222222
-- Or you can just use H160.from_H256 and pass third element of topics array

-- Takes non-hex encoded data and returns value
SELECT ERC20.transfer_value('00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000001d688');
-- 120456

SELECT ERC721.transfer_token('00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000001d688');
-- 120456
```

### Sushiswap / Uniswap

```sql
-- Get abi encoded event hashes
SELECT Sushiswap.swap_abi();
-- 0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822

SELECT Sushiswap.sync_abi();
-- 0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1

SELECT Uniswap.swap_abi();
-- 0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67

-- Takes non-hex encoded data and returns swap type
SELECT Sushiswap.swap_type('00..');
SELECT Uniswap.swap_type('00..');
-- 0 (BUY) or 1 (SELL)

-- Pair: BASE / QUOTE
-- Takes non-hex encoded data and returns swap amount
SELECT Sushiswap.swap_base_amount('00..');
SELECT Sushiswap.swap_quote_amount('00..');

SELECT Uniswap.swap_base_amount('00..');
SELECT Uniswap.swap_quote_amount('00..');

-- Takes non-hex encoded data and returns reserve
SELECT Sushiswap.sync_base_reserve('00..');
SELECT Sushiswap.sync_quote_reserve('00..');

SELECT Uniswap.sync_base_reserve('00..');
SELECT Uniswap.sync_quote_reserve('00..');

-- Takes non-hex encoded data and base decimals / quote decimals, returns price
SELECT Sushiswap.sync_price('00..', 18, 18);

-- Takes non-hex encoded data of swap and base/quote decimals, returns price
-- Uniswap does not have sync event but we can get the same result using swap event
SELECT Uniswap.sync_price('00..', 18, 18);
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
