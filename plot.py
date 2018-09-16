#!/usr/bin/env python
import matplotlib.pylab as plt
import numpy as np
import sys
data=np.loadtxt(sys.argv[1])
if data.shape[1]==2:
    plt.plot(data[:,0],data[:,1])
elif data.shape[1]==3:
    plt.plot(data[:,0],data[:,1])
    plt.plot(data[:,0],data[:,2])
plt.show()
