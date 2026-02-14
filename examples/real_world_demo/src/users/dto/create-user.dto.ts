export class CreateUserDto {
  name: string;
  email: string;
}

export interface User {
  id: number;
  name: string;
  email: string;
  isActive: boolean;
}
