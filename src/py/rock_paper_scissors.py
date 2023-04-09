#!python3
# じゃんけんプログラム
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


# 勝敗判定
def determine_winner(player, computer):
    if player == Hand.グー and computer == Hand.チョキ \
            or player == Hand.パー and computer == Hand.グー \
            or player == Hand.チョキ and computer == Hand.パー:
        return Result.WIN
    elif computer == Hand.グー and player == Hand.チョキ \
            or computer == Hand.パー and player == Hand.グー \
            or computer == Hand.チョキ and player == Hand.パー:
        return Result.LOSE
    else:
        return Result.DRAW


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

