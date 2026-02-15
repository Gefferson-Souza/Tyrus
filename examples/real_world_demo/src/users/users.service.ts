import { Injectable } from "@nestjs/common";
import { CreateUserDto, User } from "./dto/create-user.dto";

@Injectable()
export class UsersService {
  private users: User[] = [];
  private idCounter: number = 1;

  create(createUserDto: CreateUserDto): User {
    const newUser: User = {
      id: this.idCounter,
      name: createUserDto.name,
      email: createUserDto.email,
      isActive: true,
    };
    this.idCounter = this.idCounter + 1;
    this.users.push(newUser);
    return newUser;
  }

  findAll(): User[] {
    return this.users;
  }

  findOne(id: number): User | null {
    // Basic imperative finds since .find might verify closure support
    // But map/filter/reduce are supported.
    // Let's use basic iteration or filter.
    const found = this.users.filter((u) => u.id === id);
    if (found.length > 0) {
      return found[0];
    }
    return null;
  }
}
