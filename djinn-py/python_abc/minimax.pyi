from abc import ABC, abstractmethod
from typing import Generic, List, Self, TypeVar

# Define a generic type for Action
A = TypeVar('A')

class MinimaxState(ABC, Generic[A]):
    @abstractmethod
    def is_terminal(self) -> bool:
        """Returns True if the state is a terminal (end) state; otherwise, False."""
        pass

    @abstractmethod
    def heuristic_value(self) -> float:
        """Calculates and returns the heuristic value of the state."""
        pass

    @abstractmethod
    def current_player(self) -> bool:
        """Returns True for the maximizing player, and False for the minimizing player."""
        pass

    @abstractmethod
    def actions(self) -> List[A]:
        """Returns a list of available actions from this state."""
        pass

    @abstractmethod
    def result(self, action: A) -> Self:
        """Returns a new state resulting from the specified action."""
        pass
