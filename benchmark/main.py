import asyncio
import websockets
import uuid
import json
import random
import time
from asyncio import Lock

# Configuration
NUM_PLAYERS = 3000
URI = "ws://localhost:8080/api/v1/queue/test/join"
REPEAT_COUNT = 10  # How many times each player should connect/reconnect
DELAY_BETWEEN_CONNECTIONS = (.1, .5)  # Optional delay range between connections


# Shared variables for timing
total_time = 0.0
total_requests = 0
timing_lock = Lock()

async def simulate_player(player_id: int, uri: str, repeat: int):
    global total_time, total_requests
    for i in range(repeat):
        data = {
            "players": [str(uuid.uuid4())]
        }
        message = json.dumps(data)

        try:
            async with websockets.connect(uri) as websocket:
                print(f"[Player {player_id}] Connected (Attempt {i+1})")

                start = time.time()

                await websocket.send(message)
                
                print(f"[Player {player_id}] Sent: {message}")

                response = await asyncio.wait_for(websocket.recv(), timeout=5)
                
                elapsed = time.time() - start
                print(f"[Player {player_id}] Received: {response}")

                # Update timing stats in a threadsafe manner
                async with timing_lock:
                    total_time += elapsed
                    total_requests += 1

        except asyncio.TimeoutError:
            print(f"[Player {player_id}] Timeout waiting for response.")
        except Exception as e:
            print(f"[Player {player_id}] Error: {e}")

        await asyncio.sleep(random.uniform(*DELAY_BETWEEN_CONNECTIONS))  # Optional delay

async def main():
    tasks = [
        asyncio.create_task(simulate_player(i, URI, REPEAT_COUNT))
        for i in range(NUM_PLAYERS)
    ]
    await asyncio.gather(*tasks)

    # Calculate and print average request time
    async with timing_lock:
        avg_time = total_time / total_requests if total_requests > 0 else 0
        print(f"Average request time: {avg_time:.4f} seconds over {total_requests} requests.")

if __name__ == "__main__":
    start = time.time()
    asyncio.run(main())
    print(f"Stress test completed in {time.time() - start:.2f} seconds.")
