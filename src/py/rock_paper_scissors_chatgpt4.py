#!python3
# ChatGPT4が作ったプログラム。無限ループするよ！
import random

def get_hand_name(hand):
    if hand == 0:
        return "グー"
    elif hand == 1:
        return "チョキ"
    elif hand == 2:
        return "パー"

def find_winner(players):
    max_hand = max(players, key=lambda x: x["hand"])
    winners = [player for player in players if player["hand"] == max_hand["hand"]]
    if len(winners) == 1:
        return winners
    else:
        return find_winner(winners)

def play_rock_paper_scissors(num_players):
    hands = [random.randint(0, 2) for _ in range(num_players)]
    players = [{"id": i, "hand": hand} for i, hand in enumerate(hands)]

    print("プレイヤーの手:")
    for player in players:
        print(f"プレイヤー {player['id'] + 1}: {get_hand_name(player['hand'])}")

    winners = find_winner(players)
    if len(winners) == 1:
        print(f"勝者: プレイヤー {winners[0]['id'] + 1}")
    else:
        print("引き分け")

num_players = int(input("プレイヤーの数を入力してください: "))
play_rock_paper_scissors(num_players)