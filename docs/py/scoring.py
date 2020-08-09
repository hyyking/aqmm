import matplotlib.pyplot as plt
from matplotlib import cm
import numpy as np

MIN_VAL = 0
MAX_VAL = 100

def lsmr(b, x, y):
    return b * np.log(np.exp(x/b) + np.exp(y/b))

def plot(fn, title, param):
    y, x = np.meshgrid(*([np.linspace(MIN_VAL, MAX_VAL, 100)] * 2))
    z = (fn(x, y) if param is None else fn(param, x, y))
    
    fig = plt.figure()
    ax = fig.gca(projection="3d") 
    surf = ax.plot_surface(x, y, z, cmap="BuPu", antialiased=False)
    ax.set_title(title)
    ax.axis([MIN_VAL, MAX_VAL] * 2)
    fig.colorbar(surf, shrink=0.5, aspect=5)


def main():
    B = 0.2
    plot(lsmr, f"LSMR (B: {B})", B)
 
    plt.show()

if __name__ == "__main__":
    main()
