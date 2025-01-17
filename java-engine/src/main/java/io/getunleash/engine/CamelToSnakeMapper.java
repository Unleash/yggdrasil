package io.getunleash.engine;

import java.lang.reflect.Method;

import com.sun.jna.FunctionMapper;
import com.sun.jna.NativeLibrary;

class CamelToSnakeMapper implements FunctionMapper {
    @Override
    public String getFunctionName(NativeLibrary library, Method method) {
        return convertToSnake(method.getName());
    }

    String convertToSnake(String inputName) {
        StringBuilder snakeCaseName = new StringBuilder();

        for (char c : inputName.toCharArray()) {
            if (Character.isUpperCase(c)) {
                snakeCaseName.append('_').append(Character.toLowerCase(c));
            } else {
                snakeCaseName.append(c);
            }
        }

        return snakeCaseName.toString();
    }
}