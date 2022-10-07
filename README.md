# Yuri Izmer 053501
database design on the topic: Importer of food products

## Requirements

1. Compulsory functional requirements:

    - User Authorization
    - User Management (CRUD)
    - Role System (Admin, Manager, Client, Supplier)
    - Logging

1. The application must allow user to: 

	- Register and authenticate
	- Create and edit:
		- product categories and subcategories. (Admin, Manager)
	- Create product categories. (Admin, Manager, Supplier)
	- Describe:
		- available types of products from each supplier. (Admin, Manager, Supplier)
		- details of warehouses and their contents. (Admin, Manager)
		- product requirements for each of client addresses. (Admin, Manager, Client)
	- View the following data:
		- List of suppliers with name, email and country.
		- List of products for category or subcategory.
		- List of product requierments for each client address.
		- List of awailable products per warehouse.
		- List of countries, categories and subcategories.
		- List of users.
		- List of all addresses for client.

## Entitity relationship diagram

![image](img/entities.svg)

## Entities

> Note: all fields are non-nullable except those marked as nullable

1. User

    - **user_id**: number, primary key
    - _supplier_id_: number, nullable and unique foreign key that refers to supplier
    - _client_id_: number, nullable and unique foreign key that refers to client
    - _user_role_id_: number, foreign key that refers to user role
    - name: varchar(320)
    - password: varchar(256)

1. User role - admin, manager, client, supplier, etc...

    - **user_role_id**: number, primary key
    - name: varchar(20)

1. Client - company that buys products

    - **client_id**: number, primary key
    - name: varchar(320)
    - email: varchar(320)

1. Client address - one of the client addresses

    - **client_address_id**: number, primary key
    - _client_id_: number, foreign key that refers to client
    - address: varchar

1. Product requirement - required count of product for address

    - **product_requirement_id**: number, primary key
    - _product_id_: number, foreign key that refers to product
    - _client_address_id_: number, foreign key that refers to client address
    - count: number

1. Supplier - company that sells products

    - **supplier_id**: number, primary key
    - _country_id_: number, foreign key that refers to country
    - name: varchar(320), name of supplier
    - email: varchar(320), email of supplier

1. Country

    - **country_id**: number, primary key
    - name: varchar(60), name of the country

1. Product

    - **product_id**: number, primary key
    - _supplier_id_: number, foreign key that refers to supplier
    - _subcategory_id_: number, foreign key that refers to subcategory
    - name: varchar, name of product

1. Product subcategory

    - **subcategory_id**: number, primary key
    - _category_id_: number, foreign key that refers to category
    - name: varchar, name of subcategory

1. Product category - grocery, canned food, healthy food etc...

    - **category_id**: number, primary key
    - name: varchar, name of the category

1. Warehouse - place where products can be located

    - **warehouse_id**: number, primary key
    - address: varchar, name of the category

1. Product locations
    - **product_location_id**: number, primary key
    - _warehouse_id_: number, foreign key that refers to warehouse
    - _product_id_: number, foreign key that refers to product
    - count: number
