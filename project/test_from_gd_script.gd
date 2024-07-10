extends TestClass



func _enter_tree() -> void:
	var coroutine: GodotCoroutine = test_routine()
	var ret = await coroutine.finished
	print(str(ret))
