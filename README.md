# Поиск билетов

Апи доступно по адресу http://localhost:8000 <br>

- POST http://localhost:8000/batch_insert <br>
- POST http://localhost:8000/search <br> <br>
---
Требуется установить redis сервер. <br>
По умолчанию коннектится на адрес: redis://localhost:6379 <br>
Адрес прописан в константе REDIS_ADDRESS в файле redis_con.rs <br> <br>

---
Обработка ошибок в проекте не сделана. Не хватило времени.
