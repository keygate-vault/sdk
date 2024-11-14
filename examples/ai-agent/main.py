import os
import asyncio
from pathlib import Path
import keygate_sdk
from typing import List, Callable, Any, Dict
from datetime import datetime
from anthropic import Anthropic
from dotenv import load_dotenv
import traceback

def load_environment():
    """Load environment variables from .env.local file"""
    env_path = Path('.') / '.env.local'
    load_dotenv(dotenv_path=env_path)
    
    required_vars = ['ANTHROPIC_API_KEY']
    missing_vars = [var for var in required_vars if not os.getenv(var)]
    
    if missing_vars:
        raise ValueError(f"Missing required environment variables: {', '.join(missing_vars)}")

class ICPAgent:
    """An AI agent with ICP wallet capabilities powered by KeygateSDK and Claude."""
    
    def __init__(
        self,
        name: str,
        instructions: str,
        functions: List[Callable],
        identity_path: str = "identity.pem",
        keygate_url: str = "http://localhost:4943"
    ):
        self.name = name
        self.instructions = instructions
        self.functions = {
            "get_balance": self.get_balance,
            "get_wallet_address": self.get_wallet_address,
            "create_wallet": self.create_wallet,
            "execute_transaction": self.execute_transaction
        }
        self.identity_path = identity_path
        self.keygate_url = keygate_url
        self.keygate = None
        self.wallet_id = None
        self.anthropic = Anthropic(api_key=os.getenv("ANTHROPIC_API_KEY"))
        
    async def initialize(self):
        """Initialize the KeygateSDK client and create a wallet."""
        self.keygate = keygate_sdk.PyKeygateClient(
            identity_path=self.identity_path,
            url=self.keygate_url
        )
        await self.keygate.init()
        self.wallet_id = await self.keygate.create_wallet()
        
    async def get_wallet_address(self) -> str:
        """Get the ICP address for the agent's wallet."""
        if not self.wallet_id:
            raise ValueError("Agent not initialized. Call initialize() first.")
        return await self.keygate.get_icp_address(self.wallet_id)
        
    async def get_balance(self) -> float:
        """Get the ICP balance of the agent's wallet."""
        if not self.wallet_id:
            raise ValueError("Agent not initialized. Call initialize() first.")
        return await self.keygate.get_icp_balance(self.wallet_id)

    async def create_wallet(self) -> str:
        """Create a new ICP wallet."""
        return await self.keygate.create_wallet()
    
    async def execute_transaction(self, recipient_address: str, amount: float) -> str:
        """Execute an ICP transaction to a recipient address."""
        if not self.wallet_id:
            raise ValueError("Agent not initialized. Call initialize() first.")
        return await self.keygate.execute_transaction(self.wallet_id, recipient_address, amount)

    def format_functions_for_claude(self) -> str:
        """Format available functions as a string for Claude's context."""
        functions_desc = "Available functions:\n\n"
        for name, func in self.functions.items():
            functions_desc += f"{name}: {func.__doc__}\n\n"
        return functions_desc

    async def process_message(self, message: str) -> str:
        """Process a message using Claude and execute any requested functions."""
        try:
            # Create the message for Claude including available functions
            system_prompt = f"""You are {self.name}, an AI agent with an ICP wallet.
{self.instructions}

{self.format_functions_for_claude()}

To execute a function, respond with XML tags like this:
<function>function_name</function>

For example:
<function>get_balance</function>

Only call one function at a time. If no function needs to be called, respond normally. If you can't do something, say so and be concise and straight to the point. Don't talk more than necessary.
"""
            
            # Get response from Claude
            response = self.anthropic.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=1024,
                temperature=0,
                system=system_prompt,
                messages=[
                    {"role": "user", "content": message}
                ]
            )
            
            content = response.content[0].text
            
            # Check if Claude wants to execute a function
            if "<function>" in content and "</function>" in content:
                start_idx = content.find("<function>") + len("<function>")
                end_idx = content.find("</function>")
                func_name = content[start_idx:end_idx].strip()
                
                if func_name in self.functions:
                    # Execute the function
                    func_result = await self.functions[func_name]()
                    
                    # Get final response from Claude with the function result
                    final_response = self.anthropic.messages.create(
                        model="claude-3-sonnet-20240229",
                        max_tokens=1024,
                        temperature=0,
                        system=system_prompt,
                        messages=[
                            {"role": "user", "content": message},
                            {"role": "assistant", "content": content},
                            {"role": "user", "content": f"Function result: {func_result}"}
                        ]
                    )
                    return final_response.content[0].text
                
            return content
            
        except Exception as e:
            return f"Error processing message: {traceback.format_exc()}"

class AutoTasks:
    """Collection of automated tasks for the ICP agent."""
    
    @staticmethod
    async def monitor_balance(agent: ICPAgent, min_balance: float = 5.0):
        """Monitor wallet balance and alert if it falls below threshold."""
        balance = await agent.get_balance()
        if balance < min_balance:
            print(f"⚠️ Low balance alert: {balance} ICP")
            # You could add more alert mechanisms here (email, webhook, etc.)
        return balance

async def run_chat_mode():
    """Run the agent in interactive chat mode."""
    instructions = """
    You are an AI assistant that helps users manage their ICP wallet. You can:
    1. Check wallet balance
    2. Get wallet address
    3. Process transactions
    
    Always be helpful and security-conscious when handling financial operations.
    If asked about cryptocurrency prices or market data, explain that you don't have access to real-time market data.
    """
    
    # Create the agent
    agent = ICPAgent(
        name="ICP Assistant",
        instructions=instructions,
        functions=[]
    )
    
    # Initialize the agent
    await agent.initialize()
    
    print(f"🤖 {agent.name} initialized!")
    print(f"💳 Wallet created: {agent.wallet_id}")
    
    while True:
        user_input = input("\nYou: ")
        if user_input.lower() in ['quit', 'exit', 'bye']:
            break
            
        response = await agent.process_message(user_input)
        print(f"\n🤖 {agent.name}: {response}")

async def run_autonomous_mode(instructions: str, check_interval: int = 60):
    """Run the agent in autonomous mode with specific instructions."""
    agent = ICPAgent(
        name="Autonomous ICP Agent",
        instructions=instructions,
        functions=[
            ICPAgent.get_balance,
            ICPAgent.get_wallet_address,
            ICPAgent.execute_transaction,
            lambda: AutoTasks.monitor_balance(agent)
        ]
    )
    
    await agent.initialize()
    
    print(f"🤖 Autonomous agent initialized with wallet {agent.wallet_id}")
    
    while True:
        # Process the autonomous instructions
        response = await agent.process_message(
            f"Current time: {datetime.now()}. Please perform your routine checks and operations."
        )
        print(f"\n🤖 Autonomous action: {response}")
        
        # Wait for the next check interval
        await asyncio.sleep(check_interval)

if __name__ == "__main__":
    # Load environment variables
    load_environment()

    # Run in chat mode
    asyncio.run(run_chat_mode())