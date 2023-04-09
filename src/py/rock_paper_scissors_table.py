#!python3
# じゃんけんプログラム: テーブル引き
from enum import Enum, auto
import random


# グー、チョキ、パーの手
class Hand(Enum):
    グー = 0
    チョキ = 1
    パー = 2


# 勝敗
class Result(Enum):
    WIN = auto()
    LOSE = auto()
    DRAW = auto()


TABLE = [
    # Computer グー, チョキ    ,  パー
    [Result.DRAW,    Result.WIN,  Result.LOSE],  # Player グー
    [Result.LOSE,    Result.DRAW, Result.WIN],   # Player チョキ
    [Result.WIN,     Result.LOSE, Result.DRAW]   # Player パー
]


# 勝敗判定
def determine_winner(player, computer):
    return TABLE[player.value][computer.value]


# プレイヤーの入力を取得
def get_player_hand():
    while True:
        try:
            player_hand = int(input("手を入力してください (0: グー, 1: チョキ, 2: パー): "))
            player_hand = Hand(player_hand)
            return player_hand
        except (ValueError, KeyError):
            print("不正な入力です。 0: グー, 1: チョキ, 2: パー のいずれかを入力してください。")


# 乱数で手を作成
def get_computer_hand():
    return random.choice(list(Hand))


# ゲームの実行
player_hand = get_player_hand()
computer_hand = get_computer_hand()
result = determine_winner(player_hand, computer_hand)

print("あなたの手:", player_hand.name)
print("コンピューターの手:", computer_hand.name)
if result == Result.WIN:
    print("あなたの勝ちです")
elif result == Result.LOSE:
    print("コンピューターの勝ちです")
else:
    print("引き分けです")
