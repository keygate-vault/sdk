import keygate_sdk
import asyncio

async def main():
    keygate = keygate_sdk.PyKeygateClient(identity_path="identity.pem", url="https://ic0.app")
    await keygate.init()
    print("Initialized Keygate")
    print("--------------------------------")

asyncio.run(main())