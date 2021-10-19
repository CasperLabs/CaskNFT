# CaskNFT Contract

The CaskNFT contract was designed  to provide [Metacask](https://metacask.com/) a mechanism on minting a non-fungible token accurately representing the descriptive + administrative properties of a physical asset "whiskey cask"  on the Casper Network through Metacask's NFT marketplace Dapp.  Its design allows the CaskNFT contract to interact with other contracts in the marketplace ecosystem that facilitate the auction mechanics.   
1)  the [Auction contract](https://github.com/CasperLabs/casper-private-auction) to emulate the real-life operational procedures in administering the ownership rights, purchase, sale, transfer of ownership and funds between marketplace actors in English + Dutch style auctions.    
2) all while complying with regulatory requirements on identity verification standards managed by the [KYC Contract](https://github.com/CasperLabs/civic-contract).  

## Contract Data Model
| Property | Object | CLType | Description |
| --- | --- | --- | --- |
| admins | Named key | Dict(PublicKey, ()) | Admins that grant/revoke mint |
| minters | Named key | Dict(PublicKey, ()) | Minters that mint/burn/update a Cask token |

## Endpoints
The CaskNFT contract derives default endpoints of CEP47 standard and have some additional endpoints.
They can be grouped into following topics:

### Metadata
| Name | CLType | Description |
| --- | --- | --- |
| name | String | Global name of the contract |
| symbol | String | Global symbol of the contract |
| meta | Dict(String, String) | Global metadata of the contract |
| total_supply | U256 | Total amount of tokens generated |
| balance_of | U256 | Amount of tokens that a user owns |
| owner_of | PublicKey | Key of the token owner |
| get_token_by_index | String | Id of the indexed token that a user owns |
| token_meta | Dict(String, String) | Metadata of each token |
| token_commission | Dict(String, String) | Commission data for each token |

### Token Control
| Name | Description |
| --- | --- |
| mint | Mint new tokens to the provided account (Only minters/admins) |
| mint_copies | Mint new tokens with same data to the provided account (Only minters/admins) |
| burn | Burn existing tokens from the provided account (Only minters/admins) |
| transfer | Transfer tokens to an kyc'd account (Only owners) |
| transfer_from | Transfer tokens from an account to another one (Only admins) |
| set_token_meta | Set metadata of an existing token (Only minters/admins) |
| update_token_meta | Update metadata of an existing token (Only minters/admins) |
| update_token_commission | Set commission of an existing token (Only minters/admins) |

### Access Management
| Name | Description |
| --- | --- |
| grant_minter | Grant the minter role to the provided account (Only admins) |
| revoke_minter | Revoke the minter role from the provided account (Only admins) |
| grant_admin | Grant the admin role to the provided account (Only admins) |
| revoke_admin | Revoke the admin role from the provided account (Only admins) |

## Install
Make sure the `wasm32-unknown-unknown` target is installed.
```
make prepare
```

### Build the Contract
```
make build-contracts
```

### Test
```
make test
```
As the CaskNFT contract interacts with other contracts, you will need to have the KYC Contract - **"civic.wasm"** located in `tests/wasm` folder to test successfully.
