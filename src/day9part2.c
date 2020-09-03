#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>

typedef struct node_t {
	unsigned long value;

	struct node_t* left;
	struct node_t* right;
} Node;

Node* move_left(Node* node, size_t count) {
	while(count--) {
		node = node->left;
	}

	return node;
}

Node* move_right(Node* node, size_t count) {
	while(count--) {
		node = node->right;
	}

	return node;
}

void insert_left(Node* root, Node* new_node) {
	Node* old_left = root->left;

	new_node->left = old_left;
	new_node->right = root;

	root->left = new_node;
	old_left->right = new_node;
}

void insert_right(Node* root, Node* new_node) {
	insert_left(root->right, new_node);
}

void remove_node(Node* target) {
	target->left->right = target->right;
	target->right->left = target->left;

	target->left = 0;
	target->right = 0;
}

/**
 * Main function. Reads all of stdin into a buffer, calls solve()
 */
int main() {
	Node root_node;
	root_node.value = 0;
	root_node.left = &root_node;
	root_node.right = &root_node;

	const static unsigned long num_players = 459;
	const static unsigned long num_marbles = 7210300;

	unsigned long scores[num_players] = {0};
	size_t current_player = 0;

	Node* new_node = 0;
	Node* current_node = &root_node;

	for(
		unsigned long current_marble = 1;
		current_marble <= num_marbles;
		++current_marble, (current_player = (current_player + 1) % num_players)
	) {
		// Add the marble
		if(current_marble % 23 != 0) {
			// If we have a dead node, reuse it
			new_node = new_node ? new_node : malloc(sizeof(Node));
			new_node->value = current_marble;

			// Insert it
			insert_left(move_left(current_node, 1), new_node);

			// This is now the current node
			current_node = new_node;
			new_node = 0;
		// Remove + score
		} else {
			// Find the score marble
			Node* score_node = move_right(current_node, 7);
			current_node = move_left(score_node, 1);

			// Remove it
			remove_node(score_node);

			// Score it
			scores[current_player] += (score_node->value + current_marble);

			// Reuse the node
			new_node = score_node;
		}
	}

	unsigned long best_score = 0;

	for(size_t player = 0; player < num_players; ++player) {
		best_score = scores[player] > best_score ? scores[player] : best_score;
	}

	printf("%lu\n", best_score);
}
