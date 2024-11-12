from abc import ABC, abstractmethod
from typing import Optional, Tuple, List

class Game(ABC):
    @abstractmethod
    def name(self) -> str:
        """Returns the name of the game."""
        pass

    @abstractmethod
    def thumbnail(self) -> str:
        """Returns an 11x5 ASCII thumbnail for the game."""
        pass

    @abstractmethod
    def display(self) -> str:
        """Returns a string representation of the game state for display."""
        pass

    @abstractmethod
    def display_size(self) -> Tuple[int, int]:
        """Returns the display size as a tuple (width, height)."""
        pass

    @abstractmethod
    def move_history(self) -> List[str]:
        """Returns the move history of the current game."""
        pass

    @abstractmethod
    def win_state(self) -> Optional[bool]:
        """Returns the win state, returning True for a decisive win, False for a draw, and None if the game is in progress."""
        pass

    @abstractmethod
    def is_valid_move(self, move: str) -> bool:
        """Checks if the given move is valid."""
        pass

    @abstractmethod
    def play_move(self, move: str) -> None:
        """Play the given move for the current players turn."""
        pass

    @abstractmethod
    def computer_move(self) -> str:
        """Determines and returns a move for the computer."""
        pass

    @abstractmethod
    def reset(self) -> None:
        """Resets the game to the initial state."""
        pass