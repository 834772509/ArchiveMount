using System;
using System.Collections.Concurrent;
using System.Collections.Generic;

namespace WinAVFS.Core
{
    public class ConcurrentObjectPool<T>
    {
        private readonly ConcurrentBag<T> pool;
        private readonly Func<T> factory;

        public ConcurrentObjectPool(Func<T> factory)
        {
            this.pool = new ConcurrentBag<T>();
            this.factory = factory;
        }

        public T Get()
        {
            return this.pool.TryTake(out var item) ? item : factory();
        }

        public void Put(T item)
        {
            this.pool.Add(item);
        }

        public T[] GetAll()
        {
            return this.pool.ToArray();
        }
    }
}