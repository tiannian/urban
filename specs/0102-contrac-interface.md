Position NFT:

```text
tokenOfOwnerByIndex(address,uint256)returns(uint256)
```

PositionManager:

```solidity
struct DecreaseLiquidityParams {
    uint256 tokenId;
    uint128 liquidity;
    uint256 amount0Min;
    uint256 amount1Min;
    uint256 deadline;
}

function decreaseLiquidity(DecreaseLiquidityParams calldata params) external returns (uint256 amount0, uint256 amount1);
// decreaseLiquidity(uint256,uint128,uint256,uint256,uint256)returns(uint256,uint256)

struct CollectParams {
    uint256 tokenId;
    address recipient;
    uint128 amount0Max;
    uint128 amount1Max;
}

// collect(uint256,address,uint128,uint128)returns(uint256,uint256)
function collect(CollectParams calldata params) external payable returns (uint256 amount0, uint256 amount1);

function positions(uint256 tokenId)
external
view
returns (
    uint96 nonce,
    address operator,
    address token0,
    address token1,
    uint24 fee,
    int24 tickLower,
    int24 tickUpper,
    uint128 liquidity,
    uint256 feeGrowthInside0LastX128,
    uint256 feeGrowthInside1LastX128,
    uint128 tokensOwed0,
    uint128 tokensOwed1
);
```
