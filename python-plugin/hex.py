from collections import deque
from typing import Tuple

SIZE = 11

class Hex:
    board = [['.' for _ in range(SIZE)] for _ in range(SIZE)]
    current_player = 'X'  # Player 'X' starts
    move_list = []

    def name(self):
        return "Hex (returned from Python)"

    def thumbnail(self):
        return "TODO"


    def display(self):
        board = "  " + " ".join([chr(ord('A') + i) for i in range(SIZE)]) + "\n"
        for row in range(SIZE):
            board += " " * row + f"{row + 1:2} " + " ".join(self.board[row]) + "\n"
        return board

    def display_size(self):
        return (32, 12)

    def move_history(self):
        return self.move_list

    def win_state(self):
            if self.__check_win():
                return True
            else:
                return None

    def switch_player(self):
        self.current_player = 'O' if self.current_player == 'X' else 'X'

    def is_valid_move(self, move: str | Tuple[int, int]):
        row = col = None
        if type(move) == str:
            try:
                if len(move) < 2:
                    return False
                col_char = move[0].upper()
                row_num = int(move[1:]) - 1
                col = ord(col_char) - ord('A')
                row = row_num
            except (IndexError, ValueError):
                return False
        else:
            row = move[0]
            col = move[1]

        return (0 <= row < SIZE) and (0 <= col < SIZE) and (self.board[row][col] == '.')

    def play_move(self, move):
        move = move.upper()
        try:
            col_char = move[0]
            row_num = int(move[1:]) - 1
            col = ord(col_char) - ord('A')
            row = row_num
        except (IndexError, ValueError):
            raise Exception("Invalid move format. Use format like A1, B3, etc.")


        if self.is_valid_move((row, col)):
            self.move_list.append(move)
            self.board[row][col] = self.current_player
            self.switch_player()
        else:
            raise Exception("Invalid move. Cell is either occupied or out of bounds.")

    def computer_move(self):
        self.x += 1
        return f"A{self.x}"


    def __get_neighbors(self, row, col):
        directions = [(-1, 0), (-1, 1), (0, -1),
                      (0, 1), (1, -1), (1, 0)]
        neighbors = []
        for dr, dc in directions:
            r, c = row + dr, col + dc
            if 0 <= r < SIZE and 0 <= c < SIZE:
                neighbors.append((r, c))
        return neighbors

    def __check_win(self):
        visited = set()
        queue = deque()

        if self.current_player == 'X':
            # Player X connects North to South
            for col in range(SIZE):
                if self.board[0][col] == 'X':
                    queue.append((0, col))
                    visited.add((0, col))
            target_row = SIZE - 1
            while queue:
                row, col = queue.popleft()
                if row == target_row:
                    return True
                for neighbor in self.__get_neighbors(row, col):
                    if neighbor not in visited and self.board[neighbor[0]][neighbor[1]] == 'X':
                        visited.add(neighbor)
                        queue.append(neighbor)
        else:
            # Player O connects East to West
            for row in range(SIZE):
                if self.board[row][0] == 'O':
                    queue.append((row, 0))
                    visited.add((row, 0))
            target_col = SIZE - 1
            while queue:
                row, col = queue.popleft()
                if col == target_col:
                    return True
                for neighbor in self.__get_neighbors(row, col):
                    if neighbor not in visited and self.board[neighbor[0]][neighbor[1]] == 'O':
                        visited.add(neighbor)
                        queue.append(neighbor)
        return False
