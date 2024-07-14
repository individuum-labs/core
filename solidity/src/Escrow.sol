// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;
import "lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import "lib/openzeppelin-contracts/contracts/utils/cryptography/SignatureChecker.sol";

address constant USDC = 0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d;

contract Escrow {
    uint public reward_rate;
    uint public total_funds;
    string public required_string;
    mapping(uint => uint) public contributions;

    constructor(uint _reward_rate, uint _total_funds, string memory _required_string) {
        reward_rate = _reward_rate;
        total_funds = _total_funds;
        required_string = _required_string;

        IERC20(USDC).transferFrom(msg.sender, address(this), _total_funds);
    }

    function contains(string memory what, string memory where)  pure private {
        bytes memory whatBytes = bytes (what);
        bytes memory whereBytes = bytes (where);

        require(whereBytes.length >= whatBytes.length);

        bool found = false;
        for (uint i = 0; i <= whereBytes.length - whatBytes.length; i++) {
            bool flag = true;
            for (uint j = 0; j < whatBytes.length; j++)
                if (whereBytes [i + j] != whatBytes [j]) {
                    flag = false;
                    break;
                }
            if (flag) {
                found = true;
                break;
            }
        }
        require (found);
    }

    function process_post(uint likes, uint post_id,  string calldata full_text, bytes calldata signature) external {
        bytes memory data = abi.encode(likes, post_id, full_text);
        SignatureChecker.isValidSignatureNow(0x49Dc27B14CfEe893e4AC9E47984Ca6B2Dccd7A2E, keccak256(data), signature);
        contains(required_string, full_text);
        uint claimed_likes = contributions[post_id];
        uint likes_to_claim = likes - claimed_likes;
        contributions[post_id] = likes;
        IERC20(USDC).transfer(msg.sender, reward_rate*likes_to_claim);
    }

}