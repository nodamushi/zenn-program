#!python3
# じゃんけんプログラム: bit 演算
from enum import Enum, auto
from typing import List
import random


# 勝敗
class Result(Enum):
    WIN = auto()
    LOSE = auto()
    DRAW = auto()


# グー、チョキ、パーの手
class Hand(Enum):
    グー = 18
    チョキ = 4
    パー = 9


# 勝敗判定
def determine_winner(players: List[Hand]):
    # 場に出されている手を集める
    hands = 0
    for player in players:
        hands |= player.value
    # 勝利手の決定
    win_hand = hands & (2 + 4 + 8) & (hands >> 1) & ~(hands << 1)
    # 結果
    results = []
    for player in players:
        if win_hand == 0:
            results.append(Result.DRAW)
        elif (win_hand & player.value) != 0:
            results.append(Result.WIN)
        else:
            results.append(Result.LOSE)
    return results


# 乱数で手を作成
def get_computer_hand():
    return random.choice(list(Hand))


# ゲームの実行
size = 4
players = []
for x in range(size):
    hand = get_computer_hand()
    players.append(hand)
    print(f"コンピューター{x}の手: {hand.name}")

results = determine_winner(players)
for x in range(size):
    hand = players[x]
    result = results[x]
    if result == Result.WIN:
        print(f"コンピューター{x}の手: 勝ち {hand.name}")
    elif result == Result.LOSE:
        print(f"コンピューター{x}の手: 負け {hand.name}")
    else:
        print(f"コンピューター{x}の手: 引き分け {hand.name}")

