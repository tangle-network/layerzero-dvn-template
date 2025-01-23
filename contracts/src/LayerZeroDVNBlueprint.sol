// SPDX-License-Identifier: UNLICENSE
pragma solidity >=0.8.13;

import "./tnt-core/BlueprintServiceManagerBase.sol";

/**
 * @title HelloBlueprint
 * @dev This contract is an example of a service blueprint that provides a single service.
 */
contract LayerZeroDVNBlueprint is BlueprintServiceManagerBase {
        /// @inheritdoc IBlueprintServiceManager
    function onRegister(
        ServiceOperators.OperatorPreferences calldata operator,
        bytes calldata registrationInputs
    )
        override
        external
        payable
        virtual
        onlyFromMaster
    { }

    /// @inheritdoc IBlueprintServiceManager
    function onUnregister(ServiceOperators.OperatorPreferences calldata operator) override external virtual onlyFromMaster { }

    /// @inheritdoc IBlueprintServiceManager
    function onUpdatePriceTargets(ServiceOperators.OperatorPreferences calldata operator)
        override
        external
        payable
        virtual
        onlyFromMaster
    { }

    /// @inheritdoc IBlueprintServiceManager
    function onRequest(ServiceOperators.RequestParams calldata params) override external payable virtual onlyFromMaster { }

    /// @inheritdoc IBlueprintServiceManager
    function onApprove(
        ServiceOperators.OperatorPreferences calldata operator,
        uint64 requestId,
        uint8 restakingPercent
    )
        override
        external
        payable
        virtual
        onlyFromMaster
    { }

    /// @inheritdoc IBlueprintServiceManager
    function onReject(
        ServiceOperators.OperatorPreferences calldata operator,
        uint64 requestId
    )
        override
        external
        virtual
        onlyFromMaster
    { }

    /// @inheritdoc IBlueprintServiceManager
    function onServiceInitialized(
        uint64 requestId,
        uint64 serviceId,
        address owner,
        address[] calldata permittedCallers,
        uint64 ttl
    )
        override
        external
        virtual
        onlyFromMaster
    { }

    /// @inheritdoc IBlueprintServiceManager
    function onJobCall(
        uint64 serviceId,
        uint8 job,
        uint64 jobCallId,
        bytes calldata inputs
    )
        override
        external
        payable
        virtual
        onlyFromMaster
    { }

    /// @inheritdoc IBlueprintServiceManager
    function onJobResult(
        uint64 serviceId,
        uint8 job,
        uint64 jobCallId,
        ServiceOperators.OperatorPreferences calldata operator,
        bytes calldata inputs,
        bytes calldata outputs
    )
        override
        external
        payable
        virtual
        onlyFromMaster
    { }

    /// @inheritdoc IBlueprintServiceManager
    function onServiceTermination(uint64 serviceId, address owner) override external virtual onlyFromMaster { }

    /// @inheritdoc IBlueprintServiceManager
    function onUnappliedSlash(
        uint64 serviceId,
        bytes calldata offender,
        uint8 slashPercent,
        uint256 totalPayout
    )
        override
        external
        virtual
        onlyFromMaster
    { }

    /// @inheritdoc IBlueprintServiceManager
    function onSlash(
        uint64 serviceId,
        bytes calldata offender,
        uint8 slashPercent,
        uint256 totalPayout
    )
        override
        external
        virtual
        onlyFromMaster
    { }
}
