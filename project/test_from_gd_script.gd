extends TestClass



func _enter_tree() -> void:
	var coroutine: SpireCoroutine = test_routine()
	var ret = await coroutine.finished
	print("Result from GDScript: " + str(ret))
	
	var other = Node.new()
	add_child(other)
	var other_coroutine = test_from_other_node(other)
	var other_ret = await other_coroutine.finished
	print("Result from Other: " + str(other_ret))
