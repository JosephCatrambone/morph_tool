import tkinter
from enum import IntEnum
from tkinter import *
from tkinter import filedialog
from tkinter import ttk

import numpy
from PIL import Image, ImageTk

from keyframe_animation import Animation, Keyframe
from morph import morph


class PointEvent:
	POINT_MOVED = 0
	POINT_ADDED = 1
	POINT_REMOVED = 2
	POINTS_REFRESHED = 3

	def __init__(self, sender, idx, x, y):
		self.sender = sender
		self.event_type = None
		self.point_idx = idx
		self.point_x = x
		self.point_y = y


class PointAddedEvent(PointEvent):
	def __init__(self, sender, idx, x, y):
		super().__init__(sender, idx, x, y)
		self.event_type = PointEvent.POINT_ADDED


class PointMovedEvent(PointEvent):
	def __init__(self, sender, idx, x, y):
		super().__init__(sender, idx, x, y)
		self.event_type = PointEvent.POINT_MOVED


class PointDeletedEvent(PointEvent):
	def __init__(self, sender, idx, x, y):
		super().__init__(sender, idx, x, y)
		self.event_type = PointEvent.POINT_REMOVED


class MorphFrame(ttk.Frame):
	MAX_CLICK_DISTANCE = 400
	POINT_RADIUS = 5

	def __init__(self, parent):
		super().__init__(parent, padding=10)
		self.grid()

		self.selected_point_idx = None
		self.image = None
		self.image_tk = None
		self.canvas = Canvas(self, width=512, height=512)
		self.canvas.grid(row=1, column=0)
		self.canvas.bind('<Button-1>', self.handle_click)
		self.canvas.bind('<Double-Button-1>', self.handle_double_click)
		self.canvas.bind('<B1-Motion>', self.handle_drag)
		self.canvas.bind('<ButtonRelease-1>', self.finish_drag)
		self.load_button = Button(self, text="Load", command=self.load_image)
		self.load_button.grid(row=0, column=0)
		self.canvas_points = list()  # This is used to track all of the 'point objects' in the canvas.
		self.point_positions = numpy.zeros((0, 2), dtype=float)

		self.point_added_callback_listeners = list()
		self.point_moved_callback_listeners = list()
		self.point_deleted_callback_listeners = list()

	def load_image(self):
		filename = filedialog.askopenfilename(defaultextension="png")
		if not filename:
			return
		self.image = Image.open(filename)
		self.image_tk = ImageTk.PhotoImage(self.image)
		self.canvas.create_image((0, 0), anchor=NW, image=self.image_tk)

	def _get_nearest_point_idx(self, x, y, max_distance=MAX_CLICK_DISTANCE):
		min_dist = 1e100
		nearest_idx = None
		for idx in range(0, self.point_positions.shape[0]):
			# We have to iterate over these anyway, so don't worry about parallel compute.
			dx = x - self.point_positions[idx, 0]
			dy = y - self.point_positions[idx, 1]
			dist = dx*dx + dy*dy
			if dist < min_dist:
				min_dist = dist
				nearest_idx = idx
		if min_dist > max_distance:
			return None
		return nearest_idx

	def handle_click(self, event):
		x, y = event.x, event.y
		"""
		mods = {
			0x0001: 'Shift',
			0x0002: 'Caps Lock',
			0x0004: 'Control',
			0x0008: 'Left-hand Alt',
			0x0010: 'Num Lock',
			0x0080: 'Right-hand Alt',
			0x0100: 'Mouse button 1',
			0x0200: 'Mouse button 2',
			0x0400: 'Mouse button 3'
		}

		root.bind( '<Key>', lambda e: print( 'Key:', e.char, 'Mods:', mods.get( e.state, None )))
		"""
		nearest_point_idx = self._get_nearest_point_idx(x, y)
		for p in self.canvas_points:
			self.canvas.itemconfig(p, fill="blue")
		if nearest_point_idx is not None:
			self.selected_point_idx = nearest_point_idx
			self.canvas.itemconfig(self.canvas_points[self.selected_point_idx], fill="red")

	def handle_drag(self, event):
		if self.selected_point_idx is None:
			return
		x, y = event.x, event.y
		self.point_positions[self.selected_point_idx, 0] = x
		self.point_positions[self.selected_point_idx, 1] = y
		self.canvas.moveto(self.canvas_points[self.selected_point_idx], x-self.POINT_RADIUS, y-self.POINT_RADIUS)

	def finish_drag(self, event):
		if self.selected_point_idx is None:
			return
		x = self.point_positions[self.selected_point_idx, 0]
		y = self.point_positions[self.selected_point_idx, 1]
		evt = PointMovedEvent(self, self.selected_point_idx, x, y)
		for cb in self.point_moved_callback_listeners:
			cb(evt)

	def handle_double_click(self, event):
		x, y = event.x, event.y
		# TODO: Only make it if we are far enough away
		evt = PointAddedEvent(self, len(self.canvas_points), x, y)
		self._create_point(x, y)
		for cb in self.point_added_callback_listeners:
			cb(evt)

	def set_points(self, points: numpy.ndarray):
		"""Used if we're animating and the frame changes."""
		for p in self.canvas_points:
			self.canvas.delete(p)
		self.canvas_points.clear()
		self.point_positions = numpy.zeros((0, 2), dtype=float)
		for idx in range(0, points.shape[0]):
			# We could do p.move(delta)...
			self._create_point(points[idx, 0], points[idx, 1])
			#self.canvas.moveto(self.canvas_points[idx], self.point_positions[idx, 0], self.point_positions[idx, 1])
		#self.frame.update_idletasks(); do_expensive_op()
		#self.update()

	def _create_point(self, px, py):
		self.canvas_points.append(self.canvas.create_oval(px-self.POINT_RADIUS, py-self.POINT_RADIUS, px+self.POINT_RADIUS, py+self.POINT_RADIUS, fill='red', outline=""))
		self.point_positions = numpy.vstack((self.point_positions, numpy.asarray([[px, py]], dtype=float)))

	def get_num_points(self):
		assert self.point_positions.shape[0] == len(self.canvas_points)
		return len(self.canvas_points)


class App:
	def __init__(self):
		self.out_image = None
		self.out_label = None
		self.out_image_tk = None
		self.animation = Animation()
		self.animation_frame = 0
		self.morph_amount = 0.5

		root = Tk()
		frm = ttk.Frame(root, padding=10)
		frm.grid()
		frm.pack()

		self.automatically_recompute_morph = tkinter.BooleanVar()

		self.left_frame = MorphFrame(frm)
		self.left_frame.grid(row=0, column=0)
		self.right_frame = MorphFrame(frm)
		self.right_frame.grid(row=0, column=1)
		self.out_label = Label(frm)
		self.out_label.grid(row=0, column=2)

		self.left_frame.point_added_callback_listeners.append(self.on_left_adds_point)
		self.left_frame.point_moved_callback_listeners.append(self.on_left_changes_point)
		self.right_frame.point_added_callback_listeners.append(self.on_right_adds_point)
		self.right_frame.point_moved_callback_listeners.append(self.on_right_changes_point)

		morph_amount = ttk.Scale(frm, orient=HORIZONTAL, length=200, from_=0.0, to=1.0, command=self.morph_amount_changed)
		morph_amount.grid(column=0, row=2)

		ttk.Button(frm, text="Recompute Morph", command=self.recalculate_morph_output).grid(column=1, row=2)
		ttk.Checkbutton(frm, text="Automatically Refresh", onvalue=True, offvalue=False, variable=self.automatically_recompute_morph).grid(column=2, row=2)
		ttk.Button(frm, text="Quit", command=root.destroy).grid(column=3, row=2)

		self.root = root
		self.frame = frm
		# self.root.after(100, self._redraw_left)
		# self.root.update()

	def on_left_adds_point(self, evt: PointAddedEvent):
		self.animation.add_point((evt.point_x, evt.point_y), (evt.point_x, evt.point_y), 0)
		_, right, _ = self.animation.get_points(1.0, 0)
		self.right_frame.set_points(right)

	def on_right_adds_point(self, evt: PointAddedEvent):
		self.animation.add_point((evt.point_x, evt.point_y), (evt.point_x, evt.point_y), 0)
		left, _, _ = self.animation.get_points(1.0, 0)
		self.left_frame.set_points(left)

	def on_left_changes_point(self, evt: PointMovedEvent):
		self.animation.update_point((evt.point_x, evt.point_y), None, self.animation_frame, evt.point_idx)
		if self.automatically_recompute_morph:
			self.recalculate_morph_output()

	def on_right_changes_point(self, evt: PointMovedEvent):
		self.animation.update_point(None, (evt.point_x, evt.point_y), self.animation_frame, evt.point_idx)
		if self.automatically_recompute_morph:
			self.recalculate_morph_output()

	def set_output_image(self, img: Image.Image):
		self.out_image = img
		self.out_image_tk = ImageTk.PhotoImage(img)
		self.out_label.configure(width=img.width, height=img.height, image=self.out_image_tk)

	def morph_amount_changed(self, new_value: float):
		self.morph_amount = float(new_value)
		if self.automatically_recompute_morph:
			self.recalculate_morph_output()

	def recalculate_morph_output(self):
		left_image = self.left_frame.image
		right_image = self.right_frame.image
		if left_image is None or right_image is None or self.left_frame.get_num_points() < 3:
			return
		# TODO: Make sure we have at least some points.
		self.frame.update_idletasks()
		self.frame.update()
		left_points, right_points, interp_points = self.animation.get_points(self.morph_amount, self.animation_frame)
		out = morph(left_points, right_points, numpy.asarray(left_image), numpy.asarray(right_image), interp_points, self.morph_amount, left_image.width, left_image.height)
		out_image = Image.fromarray(out.astype(numpy.uint8))
		self.set_output_image(out_image)

	def run(self):
		self.root.mainloop()


def main():
	app = App()
	app.run()


if __name__ == '__main__':
	main()
