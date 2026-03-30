import logging
from pathlib import Path

import uvicorn

CALCE_AI_DIR = str(Path(__file__).resolve().parent)


def main():
    logging.basicConfig(
        level=logging.INFO,
        format="%(levelname)s:  %(name)s - %(message)s",
    )
    uvicorn.run(
        "calce_ai.app:app",
        host="0.0.0.0",
        port=35801,
        reload=True,
        reload_dirs=[CALCE_AI_DIR],
    )


if __name__ == "__main__":
    main()
