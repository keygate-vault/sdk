import keygate_sdk
import asyncio

async def main():
    keygate = keygate_sdk.PyKeygateClient(identity_path="identity.pem", url="http://localhost:63617")
    await keygate.init()
    print("Initialized Keygate")
    print("--------------------------------")

    print("Creating a wallet")
    wallet_id = await keygate.create_wallet()
    print(wallet_id)

    print("--------------------------------")
    print("Getting ICP address")
    print(await keygate.get_icp_address(wallet_id))

    print("--------------------------------")

    print("Getting ICP balance")
    print(await keygate.get_icp_balance(wallet_id))

asyncio.run(main())