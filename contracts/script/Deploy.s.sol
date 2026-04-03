// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/ProofOfClawVerifier.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address verifierAddress = vm.envAddress("RISC_ZERO_VERIFIER_ADDRESS");
        bytes32 imageId = vm.envBytes32("RISC_ZERO_IMAGE_ID");

        vm.startBroadcast(deployerPrivateKey);

        ProofOfClawVerifier proofOfClaw = new ProofOfClawVerifier(
            IRiscZeroVerifier(verifierAddress),
            imageId
        );

        console.log("ProofOfClawVerifier deployed at:", address(proofOfClaw));

        vm.stopBroadcast();
    }
}
