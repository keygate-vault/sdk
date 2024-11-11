import keygate_sdk
import asyncio

async def main():
    keygate = keygate_sdk.PyKeygateClient(identity_path="identity.pem", url="http://localhost:55174")
    await keygate.init()
    print("Initialized Keygate")
    print("--------------------------------")

    print("Creating a wallet")
    print(await keygate.create_wallet())

    print("--------------------------------")

    

asyncio.run(main())