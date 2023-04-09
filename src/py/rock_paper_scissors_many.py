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
    print("コンピューター", x, "の手:", hand.name)

results = determine_winner(players)
for x in range(size):
    hand = players[x]
    result = results[x]
    if result == Result.WIN:
        print("コンピューター", x, ": ", hand.name, " 勝ち")
    elif result == Result.LOSE:
        print("コンピューター", x, ": ", hand.name, " 負け")
    else:
        print("コンピューター", x, ": ", hand.name, " 引き分け")

