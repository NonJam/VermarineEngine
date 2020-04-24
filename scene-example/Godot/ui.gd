extends Node2D

var push = false
var pop = false

func _shadow_update(var to_print):
	self.get_node("Text").text = to_print

func _get_text_input():
	if push:
		push = false
		return self.get_node("Input").text

func _get_pop_input():
	if pop:
		pop = false
		return true

func _push_button():
	push = true

func _pop_button():
	pop = true
