// SPDX-License-Identifier: UNLICENSE
pragma solidity >=0.8.13;

import "@layerzerolabs/lz-evm-messagelib-v2/contracts/uln/interfaces/ILayerZeroDVN.sol";

contract LayerZeroDVNInstance is ILayerZeroDVN {
    function assignJob(AssignJobParam calldata _param, bytes calldata _options)
        external
        payable
        override
        returns (uint256 fee)
    {
        revert("LayerZeroDVNInstance: not implemented");
    }

    function getFee(uint32 _dstEid, uint64 _confirmations, address _sender, bytes calldata _options)
        external
        view
        override
        returns (uint256 fee)
    {
        revert("LayerZeroDVNInstance: not implemented");
    }
}
