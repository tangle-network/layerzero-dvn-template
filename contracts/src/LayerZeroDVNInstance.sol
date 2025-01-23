// SPDX-License-Identifier: LZBL-1.2
pragma solidity ^0.8.20;

import "@layerzerolabs-lz-evm-messagelib-v2/uln/interfaces/ILayerZeroDVN.sol";
import "@layerzerolabs-lz-evm-messagelib-v2/uln/interfaces/IDVNFeeLib.sol";
import "@layerzerolabs-lz-evm-messagelib-v2/uln/dvn/DVNFeeLib.sol";
import "@layerzerolabs-lz-evm-messagelib-v2/uln/dvn/adapters/DVNAdapterBase.sol";
import "@layerzerolabs-lz-evm-messagelib-v2/uln/dvn/adapters/libs/DVNAdapterMessageCodec.sol";
import "@openzeppelin-contracts/access/Ownable.sol";

/**
 * @title LayerZeroDVNInstance
 * @dev Basic DVN implementation that integrates with LayerZero ULN for verification
 */
contract LayerZeroDVNInstance is DVNAdapterBase {
    // Events
    event JobAssigned(uint32 dstEid, bytes32 payloadHash, uint64 confirmations, address sender);
    event FeeConfigSet(uint32 dstEid, uint256 baseFee);
    event MessageVerified(uint64 nonce, bytes32 payloadHash);
    event HashVerified(uint256 messageId, bytes32 hash);
    event VerifierFeePaid(uint256 fee);

    // State variables
    mapping(uint32 => uint256) public baseFees; // dstEid => base fee amount
    mapping(bytes32 => bool) public verifiedMessages; // messageId => verified status
    IDVNFeeLib public feeLib;

    // Errors
    error MessageAlreadyVerified();
    error InvalidMessageHash();
    error VerificationFailed();
    error DstEidMismatch();
    error CustomVerificationFailed();

    // Match IDVN.DstConfig types
    struct DstConfig {
        uint64 gas;
        uint16 multiplierBps;
        uint128 floorMarginUSD;
    }

    mapping(uint32 => DstConfig) public dstConfig;

    constructor(address _owner, address[] memory _admins, address payable _feeLib) DVNAdapterBase(_owner, _admins, 0) {
        feeLib = IDVNFeeLib(_feeLib);
    }

    /**
     * @notice Called by LayerZero endpoint when a new verification job is assigned
     */
    function assignJob(AssignJobParam calldata _param, bytes calldata _options)
        external
        payable
        override
        onlyAcl(_param.sender)
        returns (uint256 fee)
    {
        // Get the receive library for the destination chain
        bytes32 receiveLib = _getAndAssertReceiveLib(msg.sender, _param.dstEid);

        // Calculate fee based on destination chain and confirmations
        fee = getFee(_param.dstEid, _param.confirmations, _param.sender, _options);
        require(msg.value >= fee, "LayerZeroDVNInstance: insufficient fee");

        // Emit event for off-chain DVN to pick up the job
        emit JobAssigned(_param.dstEid, _param.payloadHash, _param.confirmations, _param.sender);

        // Return excess fee
        if (msg.value > fee) {
            (bool success,) = msg.sender.call{value: msg.value - fee}("");
            require(success, "LayerZeroDVNInstance: failed to return excess fee");
        }

        emit VerifierFeePaid(fee);
        return fee;
    }

    /**
     * @notice Verify a message from the off-chain DVN
     */
    function verifyMessageHash(bytes32 messageId, bytes calldata message)
        external
        onlyRole(ADMIN_ROLE)
        returns (uint256)
    {
        // Check if message was already verified
        if (verifiedMessages[messageId]) {
            revert MessageAlreadyVerified();
        }

        // Decode the message using DVNAdapterMessageCodec
        (address receiveLib, bytes memory packetHeader, bytes32 payloadHash) = DVNAdapterMessageCodec.decode(message);

        // Decode packet header to get source and destination info
        (uint64 nonce, uint32 srcEid, uint32 dstEid, bytes32 receiver) = _decodePacketHeader(packetHeader);

        // TODO: Add custom verification logic here before allowing ULN verification
        // This is where we can add:
        // 1. Signature verification
        // 2. Zero-knowledge proof verification
        // 3. Custom security checks
        // 4. Multi-party computation results
        // 5. Oracle verification check
        // Example:
        // if (!verifyCustomSecurity(message, payloadHash)) {
        //     revert CustomVerificationFailed();
        // }

        // Verify the message using ULN contract
        _decodeAndVerify(srcEid, message);

        // Mark message as verified
        verifiedMessages[messageId] = true;

        emit MessageVerified(nonce, payloadHash);
        emit HashVerified(uint256(messageId), payloadHash);

        return 1; // Success
    }

    /**
     * @notice Get the fee for verifying a packet
     */
    function getFee(uint32 _dstEid, uint64 _confirmations, address _sender, bytes calldata _options)
        public
        view
        override
        returns (uint256 fee)
    {
        uint256 baseFee = baseFees[_dstEid];

        IDVNFeeLib.FeeParams memory params = IDVNFeeLib.FeeParams({
            dstEid: _dstEid,
            confirmations: _confirmations,
            sender: _sender,
            quorum: 1, // Default quorum, adjust as needed
            priceFeed: address(0), // TODO: Add price feed address
            defaultMultiplierBps: 10000 // Default multiplier basis points
        });

        IDVN.DstConfig memory config = IDVN.DstConfig({
            gas: dstConfig[_dstEid].gas,
            multiplierBps: dstConfig[_dstEid].multiplierBps,
            floorMarginUSD: dstConfig[_dstEid].floorMarginUSD
        });

        uint256 dynamicFee = feeLib.getFee(params, config, _options);
        return baseFee + dynamicFee;
    }

    /**
     * @notice Decode packet header according to LayerZero format
     */
    function _decodePacketHeader(bytes memory packetHeader)
        internal
        pure
        returns (uint64 nonce, uint32 srcEid, uint32 dstEid, bytes32 receiver)
    {
        assembly {
            nonce := mload(add(packetHeader, 9)) // 8 + 64
            srcEid := mload(add(packetHeader, 13)) // 8 + 64 + 32
            dstEid := mload(add(packetHeader, 49)) // 8 + 64 + 32 + 256 + 32
            receiver := mload(add(packetHeader, 81)) // 8 + 64 + 32 + 256 + 32 + 256
        }
    }

    // Admin functions
    function setBaseFee(uint32 _dstEid, uint256 _baseFee) external onlyRole(ADMIN_ROLE) {
        baseFees[_dstEid] = _baseFee;
        emit FeeConfigSet(_dstEid, _baseFee);
    }

    function setDstConfig(uint32 _dstEid, uint64 _gas, uint16 _multiplierBps, uint128 _floorMarginUSD)
        external
        onlyRole(ADMIN_ROLE)
    {
        dstConfig[_dstEid] = DstConfig({gas: _gas, multiplierBps: _multiplierBps, floorMarginUSD: _floorMarginUSD});
    }

    function setFeeLib(address payable _feeLib) external onlyRole(ADMIN_ROLE) {
        require(_feeLib != address(0), "LayerZeroDVNInstance: invalid fee lib");
        feeLib = IDVNFeeLib(_feeLib);
    }

    function withdraw(address _to, uint256 _amount) external onlyRole(ADMIN_ROLE) {
        require(_to != address(0), "LayerZeroDVNInstance: invalid recipient");
        (bool success,) = _to.call{value: _amount}("");
        require(success, "LayerZeroDVNInstance: withdrawal failed");
    }
}
