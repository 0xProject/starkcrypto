pragma solidity ^0.6.4;

import '../primefield.sol';

contract PrimeFieldTester is PrimeField {

    event log_bytes32(bytes32 data);

    function fmul_external(uint256 a, uint256 b) external pure returns (uint256) {
        return fmul(a, b);
    }

    function fadd_external(uint256 a, uint256 b) external pure returns (uint256) {
        return fadd(a, b);
    }

    function fpow_external(uint256 a, uint256 b) external {
        emit log_bytes32(hard_cast(fpow(a, b)));
    }

    function inverse_external(uint256 a) external {
        emit log_bytes32(hard_cast(inverse(a)));
    }

    function hard_cast(uint a) internal pure returns(bytes32 b) {
        assembly {
            b := a
        }
    }
}
