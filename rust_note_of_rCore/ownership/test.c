struct student {
	int id;
	int age;
};

struct student new() {
	struct student a;
	a.id = 1;
	a.age = 12;
	printf("%p\n", &a);
	return a;
}

int main() {
	struct student a = new();
	printf("%p\n", &a);
}
