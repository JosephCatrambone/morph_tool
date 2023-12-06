
def lerp(a, b, amount: float):
	delta = b-a
	return a + (delta*amount)

"""
def lerp(a, b, coord):
    if isinstance(a, tuple):
        return tuple([lerp(c, d, coord) for c,d in zip(a,b)])
    ratio = coord - math.floor(coord)
    return int(round(a * (1.0-ratio) + b * ratio))

def bilinear(im, x, y):
    x1, y1 = int(floor(x)), int(floor(y))
    x2, y2 = x1+1, y1+1
    left = lerp(im.getpixel((x1, y1)), im.getpixel((x1, y2)), y)
    right = lerp(im.getpixel((x2, y1)), im.getpixel((x2, y2)), y)
    return lerp(left, right, x)
"""